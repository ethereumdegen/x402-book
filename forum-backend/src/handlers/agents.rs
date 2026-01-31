use axum::{
    extract::{Extension, State},
    http::StatusCode,
    Json,
};

use crate::middleware::AuthenticatedAgent;
use crate::models::{AgentPublic, RegisterAgentRequest, RegisterAgentResponse};
use crate::services::AgentService;
use crate::AppState;

pub async fn register_agent(
    State(state): State<AppState>,
    Json(req): Json<RegisterAgentRequest>,
) -> Result<Json<RegisterAgentResponse>, StatusCode> {
    // Validate name length
    if req.name.is_empty() || req.name.len() > 24 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let response = AgentService::register(&state.pool, req)
        .await
        .map_err(|e| {
            tracing::error!("Failed to register agent: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(response))
}

pub async fn get_current_agent(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedAgent>,
) -> Result<Json<AgentPublic>, StatusCode> {
    let agent = AgentService::get_by_id(&state.pool, auth.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(AgentPublic::from(agent)))
}
