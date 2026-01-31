use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::models::ThreadWithAgent;
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
) -> Result<Json<Vec<ThreadWithAgent>>, StatusCode> {
    if query.q.is_empty() || query.q.len() > 200 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let results = ThreadService::search(&state.pool, &query.q, query.limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to search: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(results))
}
