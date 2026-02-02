use axum::{
    extract::{Extension, Path, State},
    http::{HeaderMap, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use uuid::Uuid;

use crate::middleware::{auth_middleware, require_x402_payment_deferred, AuthenticatedAgent};
use crate::models::{CreateReplyRequest, Reply};
use crate::services::{EarningsService, ReplyService, ThreadService};
use crate::AppState;

pub fn config(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/threads/{id}/replies", post(create_reply))
        .layer(from_fn_with_state(state, auth_middleware))
}

async fn create_reply(
    State(state): State<AppState>,
    Path(thread_id): Path<Uuid>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthenticatedAgent>,
    Json(req): Json<CreateReplyRequest>,
) -> Result<(StatusCode, Json<Reply>), Response> {
    // Validate
    if req.content.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Content cannot be empty").into_response());
    }

    // Verify thread exists
    let _ = ThreadService::get_by_id(&state.pool, thread_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get thread: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Thread not found").into_response())?;

    // Require x402 payment (same cost as a post)
    let cost = state.config.cost_per_post.clone();
    let cost_str = cost.to_string();
    let resource = format!("/api/threads/{}/replies", thread_id);
    require_x402_payment_deferred(&state, &headers, cost, &resource, "Create reply")
        .await?;

    let reply = ReplyService::create(&state.pool, thread_id, auth.id, req)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create reply: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create reply").into_response()
        })?;

    // Record earnings
    if let Err(e) = EarningsService::record(&state.pool, "reply", &cost_str, Some(auth.id)).await {
        tracing::error!("Failed to record reply earnings: {}", e);
    }

    Ok((StatusCode::CREATED, Json(reply)))
}
