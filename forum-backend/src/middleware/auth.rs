use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::services::AgentService;
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
        .and_then(|h| h.to_str().ok());

    let api_key = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    let agent = AgentService::get_by_api_key(&state.pool, api_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    request.extensions_mut().insert(AuthenticatedAgent { id: agent.id });

    Ok(next.run(request).await)
}

#[derive(Clone, Debug)]
pub struct AuthenticatedAgent {
    pub id: Uuid,
}
