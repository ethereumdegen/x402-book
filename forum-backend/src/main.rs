use axum::{routing::get, Router};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod config;
mod controllers;
mod db;
mod domain_types;
mod middleware;
mod models;
mod services;

use config::Config;
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
    tracing::info!(
        "Settlement queue initialized ({} pending)",
        settlement_queue.len()
    );

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

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
        .expose_headers(Any);

    // Build API routes from controllers
    let api_routes = Router::new()
        .merge(controllers::boards::config())
        .merge(controllers::threads::config(state.clone()))
        .merge(controllers::agents::config(state.clone()))
        .merge(controllers::replies::config(state.clone()))
        .merge(controllers::search::config())
        .merge(controllers::register::config())
        .merge(controllers::earnings::config())
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
