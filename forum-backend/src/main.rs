use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
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
use controllers::{PostsController, RegisterController, WebController};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
    pub http_client: reqwest::Client,
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

    let state = AppState {
        pool,
        config,
        http_client,
    };

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

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

    // Determine frontend dist path (check multiple locations)
    let frontend_dist = if std::path::Path::new("./forum-frontend/dist").exists() {
        Some("./forum-frontend/dist")
    } else if std::path::Path::new("../forum-frontend/dist").exists() {
        Some("../forum-frontend/dist")
    } else if std::path::Path::new("./forum-web/dist").exists() {
        Some("./forum-web/dist")
    } else if std::path::Path::new("../forum-web/dist").exists() {
        Some("../forum-web/dist")
    } else {
        tracing::warn!("Frontend dist not found - static file serving disabled");
        None
    };

    let api_routes = Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .merge(write_routes)
        .merge(register_routes)
        .merge(posts_routes)
        .with_state(state);

    let app = if let Some(dist_path) = frontend_dist {
        let index_path = format!("{}/index.html", dist_path);
        tracing::info!("Serving frontend from: {}", dist_path);

        Router::new()
            .nest("/api", api_routes)
            .fallback_service(
                ServeDir::new(dist_path)
                    .not_found_service(ServeFile::new(index_path))
            )
            .layer(cors)
            .layer(TraceLayer::new_for_http())
    } else {
        api_routes
            .layer(cors)
            .layer(TraceLayer::new_for_http())
    };

    let addr = format!("0.0.0.0:{}", port);
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
