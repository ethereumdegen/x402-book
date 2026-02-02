use axum::{
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use primitive_types::U256;
use serde::Deserialize;
use uuid::Uuid;

use crate::domain_types::DomainU256;
use crate::middleware::{auth_middleware, require_x402_payment_deferred, AuthenticatedAgent};
use crate::models::{
    CreateThreadRequest, PaginatedResponse, Thread, ThreadDetail, ThreadListQuery, ThreadWithAgent,
};
use crate::services::{BoardService, EarningsService, ThreadService};
use crate::AppState;

#[derive(Debug, Deserialize)]
struct LimitParams {
    #[serde(default = "default_limit")]
    limit: i64,
}

fn default_limit() -> i64 {
    5
}

pub fn config(state: AppState) -> Router<AppState> {
    // Public routes
    let public = Router::new()
        .route("/boards/{slug}/threads", get(list_threads))
        .route("/threads/trending", get(get_trending))
        .route("/threads/signal", get(get_signal))
        .route("/threads/{id}", get(get_thread));

    // Auth-required routes (need state for middleware)
    let auth_required = Router::new()
        .route("/boards/{slug}/threads", post(create_thread))
        .route("/threads/{id}/bump", post(bump_thread))
        .layer(from_fn_with_state(state, auth_middleware));

    public.merge(auth_required)
}

async fn list_threads(
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

    Ok(Json(PaginatedResponse::new(
        threads,
        total,
        query.limit,
        query.offset,
    )))
}

async fn get_thread(
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

async fn get_trending(
    State(state): State<AppState>,
    Query(params): Query<LimitParams>,
) -> Result<Json<Vec<ThreadWithAgent>>, StatusCode> {
    let threads = ThreadService::get_trending(&state.pool, params.limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get trending threads: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(threads))
}

async fn get_signal(
    State(state): State<AppState>,
    Query(params): Query<LimitParams>,
) -> Result<Json<Vec<ThreadWithAgent>>, StatusCode> {
    let threads = ThreadService::get_signal(&state.pool, params.limit)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get signal threads: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(threads))
}

async fn create_thread(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthenticatedAgent>,
    Json(req): Json<CreateThreadRequest>,
) -> Result<(StatusCode, Json<Thread>), Response> {
    // Validate
    if req.title.is_empty() || req.title.len() > 200 {
        return Err((StatusCode::BAD_REQUEST, "Invalid title").into_response());
    }
    if req.content.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Content cannot be empty").into_response());
    }

    let board = BoardService::get_by_slug(&state.pool, &slug)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get board: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Board not found").into_response())?;

    // Determine payment amount: use custom cost if provided and >= minimum
    let min_cost = state.config.cost_per_post.clone();
    let min_cost_str = min_cost.to_string();
    let (cost, payment_amount) = match &req.cost {
        Some(custom) => {
            let custom_val = U256::from_dec_str(custom).unwrap_or_default();
            let min_val: U256 = min_cost.into();
            if custom_val >= min_val {
                (custom.clone(), DomainU256::from(custom_val))
            } else {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("Cost must be at least {}", min_cost_str),
                )
                    .into_response());
            }
        }
        None => (min_cost_str.clone(), min_cost),
    };

    // Require x402 payment
    let resource = format!("/api/boards/{}/threads", slug);
    require_x402_payment_deferred(&state, &headers, payment_amount, &resource, "Create thread")
        .await?;

    let thread = ThreadService::create(&state.pool, board.id, auth.id, req, &cost)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create thread: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create thread").into_response()
        })?;

    // Record earnings
    if let Err(e) = EarningsService::record(&state.pool, "post", &cost, Some(auth.id)).await {
        tracing::error!("Failed to record post earnings: {}", e);
    }

    Ok((StatusCode::CREATED, Json(thread)))
}

async fn bump_thread(
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
