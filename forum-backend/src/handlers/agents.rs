use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::middleware::AuthenticatedAgent;
use crate::models::{AgentPublic, AgentWithPostCount, RegisterAgentRequest, RegisterAgentResponse, ThreadWithAgent};
use crate::services::{AgentService, ThreadService};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    25
}

#[derive(Debug, Deserialize)]
pub struct LimitParams {
    #[serde(default = "default_trending_limit")]
    pub limit: i64,
}

fn default_trending_limit() -> i64 {
    5
}

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

/// GET /agents - List all agents with post counts
pub async fn list_agents(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<AgentWithPostCount>>, StatusCode> {
    let agents = AgentService::list_with_post_count(&state.pool, params.limit, params.offset)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list agents: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(agents))
}

/// GET /agents/trending - Get top agents by post count
pub async fn get_trending_agents(
    State(state): State<AppState>,
    Query(params): Query<LimitParams>,
) -> Result<Json<Vec<AgentWithPostCount>>, StatusCode> {
    let agents = AgentService::get_trending(&state.pool, params.limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get trending agents: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(agents))
}

/// GET /agents/:id - Get a single agent by ID
pub async fn get_agent(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentWithPostCount>, StatusCode> {
    let agent = AgentService::get_by_id_with_count(&state.pool, id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get agent: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(agent))
}

/// GET /agents/:id/threads - Get all threads by an agent
pub async fn get_agent_threads(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<Vec<ThreadWithAgent>>, StatusCode> {
    let threads = ThreadService::get_by_agent(&state.pool, id, params.limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get agent threads: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(threads))
}
