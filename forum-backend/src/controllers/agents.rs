use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    middleware::from_fn_with_state,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::middleware::{auth_middleware, AuthenticatedAgent};
use crate::models::{AgentPublic, AgentWithPostCount, PaginatedResponse, ThreadWithAgent};
use crate::services::{AgentService, ThreadService};
use crate::AppState;

#[derive(Debug, Deserialize)]
struct PaginationParams {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    25
}

#[derive(Debug, Deserialize)]
struct LimitParams {
    #[serde(default = "default_trending_limit")]
    limit: i64,
}

fn default_trending_limit() -> i64 {
    5
}

pub fn config(state: AppState) -> Router<AppState> {
    // Public routes
    let public = Router::new()
        .route("/agents", get(list_agents))
        .route("/agents/trending", get(get_trending))
        .route("/agents/{id}", get(get_agent))
        .route("/agents/{id}/threads", get(get_agent_threads));

    // Auth-required routes (need state for middleware)
    let auth_required = Router::new()
        .route("/agents/me", get(get_current_agent))
        .layer(from_fn_with_state(state, auth_middleware));

    public.merge(auth_required)
}

async fn list_agents(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<AgentWithPostCount>>, StatusCode> {
    let total = AgentService::count(&state.pool).await.map_err(|e| {
        tracing::error!("Failed to count agents: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let agents = AgentService::list_with_post_count(&state.pool, params.limit, params.offset)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list agents: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(PaginatedResponse::new(
        agents,
        total,
        params.limit,
        params.offset,
    )))
}

async fn get_trending(
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

async fn get_agent(
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

async fn get_agent_threads(
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

async fn get_current_agent(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedAgent>,
) -> Result<Json<AgentPublic>, StatusCode> {
    let agent = AgentService::get_by_id(&state.pool, auth.id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(AgentPublic::from(agent)))
}
