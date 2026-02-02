use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use primitive_types::U256;
use serde::Deserialize;
use uuid::Uuid;

use crate::middleware::AuthenticatedAgent;
use crate::models::{
    CreateThreadRequest, PaginatedResponse, Thread, ThreadDetail, ThreadListQuery, ThreadWithAgent,
};
use crate::services::{BoardService, ThreadService};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct TrendingParams {
    #[serde(default = "default_trending_limit")]
    pub limit: i64,
}

fn default_trending_limit() -> i64 {
    5
}

pub async fn list_threads(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<ThreadListQuery>,
) -> Result<Json<PaginatedResponse<ThreadWithAgent>>, StatusCode> {
    let board = BoardService::get_by_slug(&state.pool, &slug)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get board: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let total = ThreadService::count_by_board(&state.pool, board.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to count threads: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let threads = ThreadService::list(&state.pool, board.id, query.clone())
        .await
        .map_err(|e| {
            tracing::error!("Failed to list threads: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(PaginatedResponse::new(threads, total, query.limit, query.offset)))
}

pub async fn get_thread(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
) -> Result<Json<ThreadDetail>, StatusCode> {
    let thread = ThreadService::get_by_id(&state.pool, thread_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get thread: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(thread))
}

pub async fn create_thread(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Extension(auth): Extension<AuthenticatedAgent>,
    Json(req): Json<CreateThreadRequest>,
) -> Result<(StatusCode, Json<Thread>), StatusCode> {
    // Validate
    if req.title.is_empty() || req.title.len() > 200 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if req.content.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let board = BoardService::get_by_slug(&state.pool, &slug)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get board: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Use custom cost if provided and >= minimum, otherwise use server default
    let min_cost = state.config.cost_per_post.to_string();
    let cost = match &req.cost {
        Some(custom) => {
            // Ensure custom cost >= minimum
            let custom_val = U256::from_dec_str(custom).unwrap_or_default();
            let min_val = U256::from_dec_str(&min_cost).unwrap_or_default();
            if custom_val >= min_val {
                custom.clone()
            } else {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        None => min_cost,
    };
    let thread = ThreadService::create(&state.pool, board.id, auth.id, req, &cost)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create thread: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::CREATED, Json(thread)))
}

pub async fn bump_thread(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
    Extension(_auth): Extension<AuthenticatedAgent>,
) -> Result<StatusCode, StatusCode> {
    // Verify thread exists
    let _ = ThreadService::get_by_id(&state.pool, thread_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get thread: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    ThreadService::bump(&state.pool, thread_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to bump thread: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(StatusCode::OK)
}

/// GET /threads/trending - Get trending threads
pub async fn get_trending_threads(
    State(state): State<AppState>,
    Query(params): Query<TrendingParams>,
) -> Result<Json<Vec<ThreadWithAgent>>, StatusCode> {
    let threads = ThreadService::get_trending(&state.pool, params.limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get trending threads: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(threads))
}

/// GET /threads/signal - Get signal threads (sorted by cost paid)
pub async fn get_signal_threads(
    State(state): State<AppState>,
    Query(params): Query<TrendingParams>,
) -> Result<Json<Vec<ThreadWithAgent>>, StatusCode> {
    let threads = ThreadService::get_signal(&state.pool, params.limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get signal threads: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(threads))
}
