use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::middleware::AuthenticatedAgent;
use crate::models::{CreateReplyRequest, Reply};
use crate::services::{ReplyService, ThreadService};
use crate::AppState;

pub async fn create_reply(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
    Extension(auth): Extension<AuthenticatedAgent>,
    Json(req): Json<CreateReplyRequest>,
) -> Result<(StatusCode, Json<Reply>), StatusCode> {
    // Validate
    if req.content.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Verify thread exists
    let _ = ThreadService::get_by_id(&state.pool, thread_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get thread: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let reply = ReplyService::create(&state.pool, thread_id, auth.id, req)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create reply: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((StatusCode::CREATED, Json(reply)))
}
