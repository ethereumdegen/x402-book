use axum::{
    body::Body,
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::services::{erc8128_verify, AgentService};
use crate::AppState;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check for X-Forwarded-Agent header (from x402-gate)
    if let Some(agent_id) = request.headers().get("X-Forwarded-Agent") {
        let agent_id = agent_id
            .to_str()
            .map_err(|_| StatusCode::BAD_REQUEST)?;
        let agent_id: Uuid = agent_id.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

        request.extensions_mut().insert(AuthenticatedAgent { id: agent_id });
        return Ok(next.run(request).await);
    }

    // Check for Bearer token
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    if let Some(ref h) = auth_header {
        if h.starts_with("Bearer ") {
            let api_key = &h[7..];
            if let Ok(Some(agent)) = AgentService::get_by_api_key(&state.pool, api_key).await {
                request.extensions_mut().insert(AuthenticatedAgent { id: agent.id });
                return Ok(next.run(request).await);
            }
        }
    }

    // Fall back to ERC-8128 signature verification
    if erc8128_verify::has_erc8128_headers(request.headers()) {
        let method = request.method().as_str().to_string();
        let uri = request.uri().clone();
        let path = uri.path().to_string();
        let query = uri.query().map(|q| q.to_string());
        let authority = request
            .headers()
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();

        // Buffer the body for signature verification
        let (parts, body) = request.into_parts();
        let body_bytes = axum::body::to_bytes(body, 1024 * 1024)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        let identity = erc8128_verify::verify_erc8128(
            &method,
            &authority,
            &path,
            query.as_deref(),
            &body_bytes,
            &parts.headers,
        )
        .map_err(|e| {
            tracing::warn!("ERC-8128 verification failed: {}", e);
            StatusCode::UNAUTHORIZED
        })?;

        // Look up agent by wallet address (no auto-creation)
        let agent = AgentService::get_by_wallet_address(
            &state.pool,
            &identity.wallet_address.to_lowercase(),
        )
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

        // Reconstruct request with the buffered body
        let mut request = Request::from_parts(parts, Body::from(body_bytes));
        request.extensions_mut().insert(AuthenticatedAgent { id: agent.id });
        let mut response = next.run(request).await;
        response.headers_mut().insert(
            "x-erc8128-credits",
            "true".parse().unwrap(),
        );
        return Ok(response);
    }

    Err(StatusCode::UNAUTHORIZED)
}

#[derive(Clone, Debug)]
pub struct AuthenticatedAgent {
    pub id: Uuid,
}
