use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::middleware::require_x402_payment_deferred;
use crate::services::{AgentService, EarningsService};
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

/// Validate that a username only contains alphanumeric characters and underscores
fn is_valid_username(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 24
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

async fn register_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, Response> {
    // Validate username: alphanumeric and underscores only, max 24 chars
    let username = req.username.trim();
    if !is_valid_username(username) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Username must be 1-24 characters, alphanumeric and underscores only",
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
        Ok(agent_id) => {
            // Record earnings for registration (raw token value string)
            let registration_cost = state.config.cost_per_registration.to_string();
            if let Err(e) = EarningsService::record(&state.pool, "registration", &registration_cost, Some(agent_id)).await {
                tracing::error!("Failed to record registration earnings: {}", e);
            }

            Ok(Json(RegisterResponse {
                api_key,
                username: username.to_string(),
            }))
        }
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
