use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::middleware::require_x402_payment_deferred;
use crate::services::AgentService;
use crate::AppState;

use super::WebController;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub api_key: String,
    pub username: String,
}

pub struct RegisterController;

impl WebController for RegisterController {
    fn routes(state: AppState) -> Router<AppState> {
        Router::new()
            .route("/register", post(register_handler))
            .with_state(state)
    }
}

async fn register_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, Response> {
    // Validate username
    let username = req.username.trim();
    if username.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Username cannot be empty",
        )
            .into_response());
    }
    if username.len() > 24 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Username must be 24 characters or less",
        )
            .into_response());
    }

    // Check if username already exists
    if let Ok(Some(_)) = AgentService::get_by_name(&state.pool, username).await {
        return Err((
            StatusCode::CONFLICT,
            "Username already exists",
        )
            .into_response());
    }

    // Require x402 payment (deferred settlement - returns immediately after verification)
    require_x402_payment_deferred(
        &state,
        &headers,
        state.config.cost_per_registration,
        "/api/register",
        "Register agent",
    )
    .await
    .map_err(|e| e)?;

    // Create the agent
    let api_key = AgentService::generate_api_key();

    match AgentService::create(&state.pool, username, &api_key).await {
        Ok(_) => Ok(Json(RegisterResponse {
            api_key,
            username: username.to_string(),
        })),
        Err(e) => {
            tracing::error!("Failed to create agent: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create agent",
            )
                .into_response())
        }
    }
}
