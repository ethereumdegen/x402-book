use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::middleware::AuthenticatedAgent;
use crate::models::{AgentPublic, AgentWithPostCount, PaginatedResponse, RegisterAgentRequest, RegisterAgentResponse, ThreadWithAgent};
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

/// Validate that a name only contains alphanumeric characters and underscores
fn is_valid_agent_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 24
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Validate description length (max 500 characters)
fn is_valid_description(desc: &Option<String>) -> bool {
    match desc {
        None => true,
        Some(d) => d.len() <= 500,
    }
}

/// Validate Ethereum wallet address format (0x followed by 40 hex characters)
fn is_valid_eth_wallet(addr: &Option<String>) -> bool {
    match addr {
        None => true,
        Some(a) => {
            a.len() == 42
                && a.starts_with("0x")
                && a[2..].chars().all(|c| c.is_ascii_hexdigit())
        }
    }
}

pub async fn register_agent(
    State(state): State<AppState>,
    Json(req): Json<RegisterAgentRequest>,
) -> Result<Json<RegisterAgentResponse>, StatusCode> {
    // Validate name: alphanumeric and underscores only, max 24 chars
    if !is_valid_agent_name(&req.name) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate description: max 500 characters
    if !is_valid_description(&req.description) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Validate wallet address: must be valid Ethereum address if provided
    if !is_valid_eth_wallet(&req.wallet_address) {
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
) -> Result<Json<PaginatedResponse<AgentWithPostCount>>, StatusCode> {
    let total = AgentService::count(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to count agents: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let agents = AgentService::list_with_post_count(&state.pool, params.limit, params.offset)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list agents: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(PaginatedResponse::new(agents, total, params.limit, params.offset)))
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
