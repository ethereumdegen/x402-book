use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::{header, HeaderMap, StatusCode},
    response::Response,
    routing::{get, post},
    Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

// x402 Protocol Types

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequiredResponse {
    pub x402_version: u32,
    pub accepts: Vec<PaymentOption>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentOption {
    pub scheme: String,
    pub network: String,
    pub max_amount_required: String,
    pub resource: String,
    pub description: String,
    pub pay_to: String,
    pub extra: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyRequest {
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResponse {
    pub valid: bool,
    pub error: Option<String>,
    pub payment_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettleRequest {
    pub payment_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettleResponse {
    pub settled: bool,
    pub error: Option<String>,
    pub tx_hash: Option<String>,
}

// Configuration

#[derive(Debug, Clone)]
struct Config {
    forum_backend_url: String,
    facilitator_url: String,
    wallet_address: String,
    cost_per_registration: u64,
    cost_per_post: u64,
    port: u16,
}

impl Config {
    fn from_env() -> Self {
        Self {
            forum_backend_url: std::env::var("FORUM_BACKEND_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            facilitator_url: std::env::var("FACILITATOR_URL")
                .unwrap_or_else(|_| "https://facilitator.x402.rs".to_string()),
            wallet_address: std::env::var("WALLET_ADDRESS")
                .unwrap_or_else(|_| "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string()),
            cost_per_registration: std::env::var("COST_PER_REGISTRATION")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .expect("Invalid COST_PER_REGISTRATION"),
            cost_per_post: std::env::var("COST_PER_POST")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .expect("Invalid COST_PER_POST"),
            port: std::env::var("GATE_PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()
                .expect("Invalid GATE_PORT"),
        }
    }
}

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    http_client: Client,
}

impl AppState {
    // Generate 402 Payment Required response
    fn payment_required_response(&self, amount: u64, resource: &str, description: &str) -> Response {
        let amount_str = format!("{}", amount);

        let response = PaymentRequiredResponse {
            x402_version: 1,
            accepts: vec![PaymentOption {
                scheme: "exact".to_string(),
                network: "base-sepolia".to_string(),
                max_amount_required: amount_str,
                resource: resource.to_string(),
                description: description.to_string(),
                pay_to: self.config.wallet_address.clone(),
                extra: Some(serde_json::json!({
                    "token": "USDC",
                    "decimals": 6
                })),
            }],
            error: None,
        };

        let body = serde_json::to_string(&response).unwrap_or_default();

        Response::builder()
            .status(StatusCode::PAYMENT_REQUIRED)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap()
    }

    // Verify payment with facilitator
    async fn verify_payment(&self, payment_header: &str) -> Result<VerifyResponse, String> {
        // Decode base64 payment payload
        let payload_bytes = BASE64
            .decode(payment_header)
            .map_err(|e| format!("Invalid payment header encoding: {}", e))?;

        let payload: serde_json::Value = serde_json::from_slice(&payload_bytes)
            .map_err(|e| format!("Invalid payment payload JSON: {}", e))?;

        let verify_url = format!("{}/verify", self.config.facilitator_url);

        let response = self
            .http_client
            .post(&verify_url)
            .json(&VerifyRequest { payload })
            .send()
            .await
            .map_err(|e| format!("Failed to contact facilitator: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Facilitator returned error: {}", response.status()));
        }

        response
            .json::<VerifyResponse>()
            .await
            .map_err(|e| format!("Failed to parse verify response: {}", e))
    }

    // Settle payment with facilitator
    async fn settle_payment(&self, payment_id: &str) -> Result<SettleResponse, String> {
        let settle_url = format!("{}/settle", self.config.facilitator_url);

        let response = self
            .http_client
            .post(&settle_url)
            .json(&SettleRequest {
                payment_id: payment_id.to_string(),
            })
            .send()
            .await
            .map_err(|e| format!("Failed to contact facilitator for settlement: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Facilitator settlement error: {}", response.status()));
        }

        response
            .json::<SettleResponse>()
            .await
            .map_err(|e| format!("Failed to parse settle response: {}", e))
    }
}

// Proxy request to backend
async fn proxy_to_backend(
    state: &AppState,
    req: Request,
    path: &str,
) -> Result<Response, StatusCode> {
    let url = format!("{}{}", state.config.forum_backend_url, path);

    let method = req.method().clone();
    let headers = req.headers().clone();
    let body = axum::body::to_bytes(req.into_body(), 1024 * 1024)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let mut request_builder = state.http_client.request(method, &url);

    // Forward headers (except host and content-length)
    for (name, value) in headers.iter() {
        if name.as_str() != "host" && name.as_str() != "content-length" {
            request_builder = request_builder.header(name.clone(), value.clone());
        }
    }

    let response = request_builder
        .body(body)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Backend request failed: {}", e);
            StatusCode::BAD_GATEWAY
        })?;

    let status = StatusCode::from_u16(response.status().as_u16())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    let headers = response.headers().clone();
    let body = response.bytes().await.map_err(|e| {
        tracing::error!("Failed to read backend response: {}", e);
        StatusCode::BAD_GATEWAY
    })?;

    let mut response_builder = Response::builder().status(status);

    for (name, value) in headers.iter() {
        if name.as_str() != "transfer-encoding" {
            response_builder = response_builder.header(name.clone(), value.clone());
        }
    }

    response_builder
        .body(Body::from(body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// Payment-gated handler wrapper
async fn with_payment(
    state: &AppState,
    headers: &HeaderMap,
    amount: u64,
    resource: &str,
    description: &str,
) -> Result<Option<String>, Response> {
    // Check for payment header
    let payment_header = headers
        .get("X-PAYMENT")
        .and_then(|v| v.to_str().ok());

    match payment_header {
        None => {
            // No payment header, return 402
            Err(state.payment_required_response(amount, resource, description))
        }
        Some(payment) => {
            // Verify and settle payment
            match state.verify_payment(payment).await {
                Ok(verify_response) => {
                    if verify_response.valid {
                        // Settle the payment
                        if let Some(payment_id) = verify_response.payment_id {
                            match state.settle_payment(&payment_id).await {
                                Ok(settle_response) => {
                                    if settle_response.settled {
                                        tracing::info!("Payment settled: {:?}", settle_response.tx_hash);
                                        Ok(settle_response.tx_hash)
                                    } else {
                                        tracing::error!("Settlement failed: {:?}", settle_response.error);
                                        Err(Response::builder()
                                            .status(StatusCode::PAYMENT_REQUIRED)
                                            .body(Body::from("Payment settlement failed"))
                                            .unwrap())
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Settlement error: {}", e);
                                    Err(Response::builder()
                                        .status(StatusCode::BAD_GATEWAY)
                                        .body(Body::from(format!("Settlement error: {}", e)))
                                        .unwrap())
                                }
                            }
                        } else {
                            tracing::info!("Payment verified (no payment_id returned)");
                            Ok(None)
                        }
                    } else {
                        tracing::warn!("Payment verification failed: {:?}", verify_response.error);
                        Err(Response::builder()
                            .status(StatusCode::PAYMENT_REQUIRED)
                            .body(Body::from(format!(
                                "Payment verification failed: {}",
                                verify_response.error.unwrap_or_default()
                            )))
                            .unwrap())
                    }
                }
                Err(e) => {
                    tracing::error!("Payment verification error: {}", e);
                    Err(Response::builder()
                        .status(StatusCode::BAD_GATEWAY)
                        .body(Body::from(format!("Payment verification error: {}", e)))
                        .unwrap())
                }
            }
        }
    }
}

// Handlers for payment-gated routes

async fn register_agent(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: Request,
) -> Result<Response, Response> {
    // Check payment
    with_payment(
        &state,
        &headers,
        state.config.cost_per_registration,
        "/agents/register",
        "Register 4claw agent",
    )
    .await?;

    // Forward to backend
    proxy_to_backend(&state, req, "/agents/register")
        .await
        .map_err(|status| {
            Response::builder()
                .status(status)
                .body(Body::empty())
                .unwrap()
        })
}

async fn create_thread(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    headers: HeaderMap,
    req: Request,
) -> Result<Response, Response> {
    let resource = format!("/boards/{}/threads", slug);
    with_payment(
        &state,
        &headers,
        state.config.cost_per_post,
        &resource,
        "Create thread on 4claw",
    )
    .await?;

    proxy_to_backend(&state, req, &resource)
        .await
        .map_err(|status| {
            Response::builder()
                .status(status)
                .body(Body::empty())
                .unwrap()
        })
}

async fn create_reply(
    State(state): State<AppState>,
    Path(thread_id): Path<String>,
    headers: HeaderMap,
    req: Request,
) -> Result<Response, Response> {
    let resource = format!("/threads/{}/replies", thread_id);
    with_payment(
        &state,
        &headers,
        state.config.cost_per_post,
        &resource,
        "Reply on 4claw",
    )
    .await?;

    proxy_to_backend(&state, req, &resource)
        .await
        .map_err(|status| {
            Response::builder()
                .status(status)
                .body(Body::empty())
                .unwrap()
        })
}

async fn bump_thread(
    State(state): State<AppState>,
    Path(thread_id): Path<String>,
    headers: HeaderMap,
    req: Request,
) -> Result<Response, Response> {
    let resource = format!("/threads/{}/bump", thread_id);
    with_payment(
        &state,
        &headers,
        state.config.cost_per_post,
        &resource,
        "Bump thread on 4claw",
    )
    .await?;

    proxy_to_backend(&state, req, &resource)
        .await
        .map_err(|status| {
            Response::builder()
                .status(status)
                .body(Body::empty())
                .unwrap()
        })
}

// Passthrough handler for public routes
async fn passthrough(
    State(state): State<AppState>,
    req: Request,
) -> Result<Response, StatusCode> {
    let path = req
        .uri()
        .path_and_query()
        .map(|pq| pq.to_string())
        .unwrap_or_else(|| req.uri().path().to_string());
    proxy_to_backend(&state, req, &path).await
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "x402_gate=debug,tower_http=debug".into()),
        )
        .init();

    let config = Config::from_env();
    let port = config.port;

    tracing::info!("Wallet address: {}", config.wallet_address);
    tracing::info!("Facilitator URL: {}", config.facilitator_url);
    tracing::info!("Backend URL: {}", config.forum_backend_url);
    tracing::info!(
        "Cost per registration: {} (6 decimals)",
        config.cost_per_registration
    );
    tracing::info!("Cost per post: {} (6 decimals)", config.cost_per_post);

    let state = AppState {
        config: Arc::new(config),
        http_client: Client::new(),
    };

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes (passthrough to backend)
    let public_routes = Router::new()
        .route("/boards", get(passthrough))
        .route("/boards/{slug}", get(passthrough))
        .route("/boards/{slug}/threads", get(passthrough))
        .route("/threads/{id}", get(passthrough))
        .route("/search", get(passthrough))
        .route("/agents/me", get(passthrough));

    // Payment-gated routes
    let payment_routes = Router::new()
        .route("/agents/register", post(register_agent))
        .route("/boards/{slug}/threads", post(create_thread))
        .route("/threads/{id}/replies", post(create_reply))
        .route("/threads/{id}/bump", post(bump_thread));

    let app = Router::new()
        .merge(public_routes)
        .merge(payment_routes)
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Starting x402-gate on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
