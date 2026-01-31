use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

mod config;
mod db;
mod handlers;
mod middleware;
mod models;
mod services;

use config::Config;
use services::CacheService;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub cache: Arc<CacheService>,
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

    // Create cache service
    let cache = CacheService::new(&config.redis_url)
        .expect("Failed to create cache service");

    let state = AppState {
        pool,
        cache: Arc::new(cache),
    };

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/boards", get(handlers::list_boards))
        .route("/boards/:slug", get(handlers::get_board))
        .route("/boards/:slug/threads", get(handlers::list_threads))
        .route("/threads/:id", get(handlers::get_thread))
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
        .route("/boards/:slug/threads", post(handlers::create_thread))
        .route("/threads/:id/replies", post(handlers::create_reply))
        .route("/threads/:id/bump", post(handlers::bump_thread))
        .layer(from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));

    let app = Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .merge(write_routes)
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
