use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::models::BoardWithStats;
use crate::services::BoardService;
use crate::AppState;

pub async fn list_boards(
    State(state): State<AppState>,
) -> Result<Json<Vec<BoardWithStats>>, StatusCode> {
    let boards = BoardService::list(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list boards: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(boards))
}

pub async fn get_board(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<crate::models::Board>, StatusCode> {
    let board = BoardService::get_by_slug(&state.pool, &slug)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get board: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(board))
}
