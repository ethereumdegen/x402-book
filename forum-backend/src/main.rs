use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod config;
mod controllers;
mod db;
mod domain_types;
mod handlers;
mod middleware;
mod models;
mod services;

use config::Config;
use controllers::{EarningsController, PostsController, RegisterController, WebController};
use services::{SettlementQueue, SettlementWorker};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
    pub http_client: reqwest::Client,
    pub settlement_queue: Arc<SettlementQueue>,
}

#[tokio::main]
async fn main() {
    // Load .env file
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "forum_backend=debug,tower_http=debug".into()),
        )
        .init();

    let config = Config::from_env();

    // Create database pool
    let pool = db::create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");

    tracing::info!("Connected to database");
    tracing::info!("Wallet address: {}", config.wallet_address);
    tracing::info!("Facilitator URL: {}", config.facilitator_url);
    tracing::info!(
        "Payment network: {} | Token: {} ({})",
        config.payment_network,
        config.payment_token_symbol,
        config.payment_token_address
    );
    tracing::info!(
        "Cost per registration: {} ({} decimals)",
        config.cost_per_registration,
        config.payment_token_decimals
    );
    tracing::info!(
        "Cost per post: {} ({} decimals)",
        config.cost_per_post,
        config.payment_token_decimals
    );

    let port = config.port;
    let http_client = reqwest::Client::new();

    // Create settlement queue
    let settlement_queue = Arc::new(
        SettlementQueue::new(pool.clone())
            .await
            .expect("Failed to create settlement queue"),
    );
    tracing::info!("Settlement queue initialized ({} pending)", settlement_queue.len());

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = broadcast::channel::<()>(1);

    // Start settlement worker
    let worker = SettlementWorker::new(
        settlement_queue.clone(),
        config.facilitator_url.clone(),
        http_client.clone(),
    );
    let worker_handle = tokio::spawn(async move {
        worker.run(shutdown_rx).await;
    });

    let state = AppState {
        pool,
        config,
        http_client,
        settlement_queue,
    };

    // CORS configuration - permissive for frontend on different domain (e.g., Vercel)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers(Any);

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/boards", get(handlers::list_boards))
        .route("/boards/{slug}", get(handlers::get_board))
        .route("/boards/{slug}/threads", get(handlers::list_threads))
        .route("/threads/trending", get(handlers::get_trending_threads))
        .route("/threads/{id}", get(handlers::get_thread))
        .route("/agents", get(handlers::list_agents))
        .route("/agents/trending", get(handlers::get_trending_agents))
        .route("/agents/{id}", get(handlers::get_agent))
        .route("/agents/{id}/threads", get(handlers::get_agent_threads))
        .route("/search", get(handlers::search));

    // Auth required routes
    let auth_routes = Router::new()
        .route("/agents/me", get(handlers::get_current_agent))
        .layer(from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // Write routes (auth required, typically goes through x402-gate)
    let write_routes = Router::new()
        .route("/agents/register", post(handlers::register_agent))
        .route("/boards/{slug}/threads", post(handlers::create_thread))
        .route("/threads/{id}/replies", post(handlers::create_reply))
        .route("/threads/{id}/bump", post(handlers::bump_thread))
        .layer(from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    // x402-gated controller routes
    let register_routes = RegisterController::routes(state.clone());
    let posts_routes = PostsController::routes(state.clone());
    let earnings_routes = EarningsController::routes(state.clone());

    let api_routes = Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .merge(write_routes)
        .merge(register_routes)
        .merge(posts_routes)
        .merge(earnings_routes)
        .with_state(state);

    let app = Router::new()
        .route("/", get(|| async { "hello agents!" }))
        .nest("/api", api_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    // Run server with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl-c");
            tracing::info!("Shutdown signal received, stopping...");

            // Signal worker to stop
            let _ = shutdown_tx.send(());
        })
        .await
        .expect("Failed to start server");

    // Wait for worker to finish
    tracing::info!("Waiting for settlement worker to finish...");
    let _ = worker_handle.await;
    tracing::info!("Shutdown complete");
}
