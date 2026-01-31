use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::middleware::AuthenticatedAgent;
use crate::models::{
    CreateThreadRequest, Thread, ThreadDetail, ThreadListQuery, ThreadWithAgent,
};
use crate::services::{BoardService, ThreadService};
use crate::AppState;

pub async fn list_threads(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Query(query): Query<ThreadListQuery>,
) -> Result<Json<Vec<ThreadWithAgent>>, StatusCode> {
    let board = BoardService::get_by_slug(&state.pool, &slug)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get board: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let threads = ThreadService::list(&state.pool, board.id, query)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list threads: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(threads))
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

    let thread = ThreadService::create(&state.pool, board.id, auth.id, req)
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
