use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde::Serialize;

use crate::services::EarningsService;
use crate::AppState;

#[derive(Debug, Serialize)]
struct EarningsResponse {
    /// Total earnings as raw token value string (256-bit, 18 decimals)
    total: String,
    breakdown: EarningsBreakdownResponse,
    count: EarningsCountResponse,
}

#[derive(Debug, Serialize)]
struct EarningsBreakdownResponse {
    /// Registration earnings as raw token value string
    registration: String,
    /// Post earnings as raw token value string
    post: String,
}

#[derive(Debug, Serialize)]
struct EarningsCountResponse {
    registrations: i64,
    posts: i64,
}

pub fn config() -> Router<AppState> {
    Router::new().route("/earnings", get(get_earnings_handler))
}

async fn get_earnings_handler(
    State(state): State<AppState>,
) -> Json<EarningsResponse> {
    let breakdown = EarningsService::get_breakdown(&state.pool)
        .await
        .unwrap_or_else(|e| {
            tracing::error!("Failed to get earnings breakdown: {}", e);
            crate::services::EarningsBreakdown {
                total: "0".to_string(),
                registration_total: "0".to_string(),
                post_total: "0".to_string(),
                registration_count: 0,
                post_count: 0,
            }
        });

    Json(EarningsResponse {
        total: breakdown.total,
        breakdown: EarningsBreakdownResponse {
            registration: breakdown.registration_total,
            post: breakdown.post_total,
        },
        count: EarningsCountResponse {
            registrations: breakdown.registration_count,
            posts: breakdown.post_count,
        },
    })
}
