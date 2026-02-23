use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Router,
};

use crate::services::SiteService;
use crate::utils;
use crate::AppState;

pub fn config() -> Router<AppState> {
    Router::new()
        .route("/s/{slug}", get(serve_root))
        .route("/s/{slug}/", get(serve_root))
        .route("/s/{slug}/{*path}", get(serve_file))
}

async fn serve_root(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Response, StatusCode> {
    serve_site_file(&state, &slug, "index.html").await
}

async fn serve_file(
    State(state): State<AppState>,
    Path((slug, path)): Path<(String, String)>,
) -> Result<Response, StatusCode> {
    serve_site_file(&state, &slug, &path).await
}

async fn serve_site_file(state: &AppState, slug: &str, path: &str) -> Result<Response, StatusCode> {
    let site = SiteService::get_by_slug(&state.pool, slug)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get site: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    if site.status != "active" {
        return Err(StatusCode::NOT_FOUND);
    }

    // Try exact path first
    if let Some(file) = SiteService::get_file_content(&state.pool, site.id, path)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get file: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
    {
        return build_response(&file.content, &file.content_type, path);
    }

    // Directory index fallback: try path/index.html
    if !path.contains('.') {
        let index_path = if path.ends_with('/') {
            format!("{}index.html", path)
        } else {
            format!("{}/index.html", path)
        };

        if let Some(file) = SiteService::get_file_content(&state.pool, site.id, &index_path)
            .await
            .map_err(|e| {
                tracing::error!("Failed to get file: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
        {
            return build_response(&file.content, &file.content_type, &index_path);
        }
    }

    Err(StatusCode::NOT_FOUND)
}

fn build_response(content: &[u8], content_type: &str, path: &str) -> Result<Response, StatusCode> {
    let is_html = utils::is_html_file(path);
    let cache_control = if is_html {
        "public, max-age=300"
    } else {
        "public, max-age=31536000"
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, cache_control)
        .header("X-Content-Type-Options", "nosniff")
        .header("X-Frame-Options", "SAMEORIGIN")
        .body(Body::from(content.to_vec()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
