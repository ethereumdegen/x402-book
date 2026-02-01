use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::models::{AgentWithPostCount, PaginatedResponse, ThreadWithAgent};
use crate::services::{AgentService, ThreadService};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    25
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub threads: PaginatedResponse<ThreadWithAgent>,
    pub agents: Vec<AgentWithPostCount>,
}

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResponse>, StatusCode> {
    if query.q.is_empty() || query.q.len() > 200 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let total = ThreadService::search_count(&state.pool, &query.q)
        .await
        .map_err(|e| {
            tracing::error!("Failed to count search results: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let threads = ThreadService::search(&state.pool, &query.q, query.limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to search threads: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let agents = AgentService::search(&state.pool, &query.q, 10)
        .await
        .map_err(|e| {
            tracing::error!("Failed to search agents: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(SearchResponse {
        threads: PaginatedResponse::new(threads, total, query.limit, 0),
        agents,
    }))
}
