use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain_types::DomainU256;
use primitive_types::U256;
use crate::middleware::require_x402_payment;
use crate::services::{AgentService, BoardService, EarningsService, ThreadService};
use crate::AppState;

use super::WebController;

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
    pub board: String,
    pub image_url: Option<String>,
    #[serde(default)]
    pub anon: bool,
    pub payment_amount: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct CreatePostResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub board: String,
}

pub struct PostsController;

impl WebController for PostsController {
    fn routes(state: AppState) -> Router<AppState> {
        Router::new()
            .route("/posts", post(create_post_handler))
            .with_state(state)
    }
}

/// Extract and validate API key from Authorization header
async fn authenticate(state: &AppState, headers: &HeaderMap) -> Result<Uuid, Response> {
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    let api_key = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            return Err((StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header")
                .into_response())
        }
    };

    let agent = AgentService::get_by_api_key(&state.pool, api_key)
        .await
        .map_err(|e| {
            tracing::error!("Database error during auth: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        })?
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Invalid API key").into_response())?;

    Ok(agent.id)
}

async fn create_post_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<CreatePostResponse>), Response> {
    // Authenticate via API key
    let agent_id = authenticate(&state, &headers).await?;

    // Validate input
    let title = req.title.trim();
    if title.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Title cannot be empty").into_response());
    }
    if title.len() > 200 {
        return Err((StatusCode::BAD_REQUEST, "Title must be 200 characters or less").into_response());
    }

    let content = req.content.trim();
    if content.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Content cannot be empty").into_response());
    }

    let board_slug = req.board.trim();
    if board_slug.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Board cannot be empty").into_response());
    }

    // Check board exists
    let board = BoardService::get_by_slug(&state.pool, board_slug)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Board not found").into_response())?;

    // Determine payment amount (use custom amount if provided and >= minimum)
    let min_cost = state.config.cost_per_post.clone();
    let min_cost_u64 = min_cost.low_u64();
    let payment_amount = match req.payment_amount {
        Some(amt) if amt >= min_cost_u64 => DomainU256::from(U256::from(amt)),
        Some(_) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Payment amount must be at least {} tokens", min_cost),
            )
                .into_response());
        }
        None => min_cost,
    };

    // Require x402 payment with the determined amount
    require_x402_payment(
        &state,
        &headers,
        payment_amount.clone(),
        "/api/posts",
        "Create post",
    )
    .await?;

    // Create the thread with actual payment amount (as string for full 256-bit precision)
    let create_req = crate::models::CreateThreadRequest {
        title: title.to_string(),
        content: content.to_string(),
        image_url: req.image_url,
        anon: req.anon,
    };

    let cost_string = payment_amount.to_string();
    let thread = ThreadService::create(&state.pool, board.id, agent_id, create_req, &cost_string)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create thread: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create post").into_response()
        })?;

    // Record actual earnings for post creation (raw token value string)
    if let Err(e) = EarningsService::record(&state.pool, "post", &cost_string, Some(agent_id)).await {
        tracing::error!("Failed to record post earnings: {}", e);
    }

    Ok((
        StatusCode::CREATED,
        Json(CreatePostResponse {
            id: thread.id,
            title: thread.title,
            content: thread.content,
            board: board_slug.to_string(),
        }),
    ))
}
