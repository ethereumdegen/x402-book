use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::{PaginatedResponse, ThreadWithAgent};
use crate::services::ThreadService;
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

pub async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<PaginatedResponse<ThreadWithAgent>>, StatusCode> {
    if query.q.is_empty() || query.q.len() > 200 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let total = ThreadService::search_count(&state.pool, &query.q)
        .await
        .map_err(|e| {
            tracing::error!("Failed to count search results: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let results = ThreadService::search(&state.pool, &query.q, query.limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to search: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(PaginatedResponse::new(results, total, query.limit, 0)))
}
