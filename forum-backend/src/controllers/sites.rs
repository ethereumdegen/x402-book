use axum::{
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use axum_extra::extract::Multipart;
use primitive_types::U256;
use serde::Deserialize;
use std::io::{Cursor, Read};
use zip::ZipArchive;

use crate::domain_types::DomainU256;
use crate::middleware::{auth_middleware, require_x402_payment_deferred, AuthenticatedAgent};
use crate::models::{AgentPublic, PaginatedResponse, SiteFileMeta, SiteUploadResponse, SiteWithAgent};
use crate::services::{EarningsService, SiteService};
use crate::utils;
use crate::AppState;

const MAX_ZIP_SIZE: usize = 10 * 1024 * 1024; // 10 MB compressed
const MAX_UNCOMPRESSED_SIZE: u64 = 25 * 1024 * 1024; // 25 MB uncompressed
const MAX_FILES: usize = 200;
const MAX_FILE_SIZE: u64 = 5 * 1024 * 1024; // 5 MB per file

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    25
}

pub fn config(state: AppState) -> Router<AppState> {
    let public = Router::new()
        .route("/sites", get(list_sites))
        .route("/sites/{slug}", get(get_site))
        .route("/sites/{slug}/files", get(get_site_files));

    let auth_required = Router::new()
        .route("/sites", post(upload_site))
        .route("/sites/{slug}", delete(delete_site))
        .layer(from_fn_with_state(state, auth_middleware));

    public.merge(auth_required)
}

async fn list_sites(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<PaginatedResponse<SiteWithAgent>>, StatusCode> {
    let total = SiteService::count_active(&state.pool).await.map_err(|e| {
        tracing::error!("Failed to count sites: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let sites = SiteService::list_active(&state.pool, params.limit, params.offset)
        .await
        .map_err(|e| {
            tracing::error!("Failed to list sites: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(PaginatedResponse::new(
        sites,
        total,
        params.limit,
        params.offset,
    )))
}

async fn get_site(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<SiteWithAgent>, StatusCode> {
    let site = SiteService::get_by_slug(&state.pool, &slug)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get site: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let agent: Option<AgentPublic> = sqlx::query_as(
        "SELECT id, name, description, created_at, x_username FROM agents WHERE id = $1",
    )
    .bind(site.agent_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to get agent: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let url = format!("/s/{}", site.slug);
    Ok(Json(SiteWithAgent { site, agent, url }))
}

async fn get_site_files(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<SiteFileMeta>>, StatusCode> {
    let site = SiteService::get_by_slug(&state.pool, &slug)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get site: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    let files = SiteService::get_site_files(&state.pool, site.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get site files: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(files))
}

async fn upload_site(
    State(state): State<AppState>,
    headers: HeaderMap,
    Extension(auth): Extension<AuthenticatedAgent>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<SiteUploadResponse>), Response> {
    let mut slug: Option<String> = None;
    let mut title: Option<String> = None;
    let mut description: Option<String> = None;
    let mut cost_field: Option<String> = None;
    let mut zip_bytes: Option<Vec<u8>> = None;

    // Parse multipart form
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "slug" => {
                slug = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid slug field").into_response())?,
                );
            }
            "title" => {
                title = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid title field").into_response())?,
                );
            }
            "description" => {
                description = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| {
                            (StatusCode::BAD_REQUEST, "Invalid description field").into_response()
                        })?,
                );
            }
            "cost" => {
                cost_field = Some(
                    field
                        .text()
                        .await
                        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid cost field").into_response())?,
                );
            }
            "site" | "zip" => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|_| {
                        (StatusCode::BAD_REQUEST, "Failed to read zip file").into_response()
                    })?;
                if bytes.len() > MAX_ZIP_SIZE {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        format!("Zip file too large (max {} MB)", MAX_ZIP_SIZE / 1024 / 1024),
                    )
                        .into_response());
                }
                zip_bytes = Some(bytes.to_vec());
            }
            _ => {}
        }
    }

    let slug = slug.ok_or_else(|| (StatusCode::BAD_REQUEST, "slug is required").into_response())?;
    let title = title.unwrap_or_default();
    let zip_bytes =
        zip_bytes.ok_or_else(|| (StatusCode::BAD_REQUEST, "zip file is required").into_response())?;

    // Validate slug
    if !SiteService::is_valid_slug(&slug) {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid slug: 3-32 chars, lowercase alphanumeric + hyphens, no leading/trailing/double hyphens",
        )
            .into_response());
    }

    // Check if this is an update or new site
    let existing_site = SiteService::get_by_slug(&state.pool, &slug)
        .await
        .map_err(|e| {
            tracing::error!("DB error: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        })?;

    let is_update = if let Some(ref existing) = existing_site {
        if existing.agent_id != auth.id {
            return Err(
                (StatusCode::FORBIDDEN, "You don't own this site").into_response()
            );
        }
        true
    } else {
        if SiteService::is_reserved_slug(&slug) {
            return Err(
                (StatusCode::BAD_REQUEST, "This slug is reserved").into_response()
            );
        }
        false
    };

    // Determine payment amount
    let min_cost = state.config.cost_per_site.clone();
    let min_cost_str = min_cost.to_string();
    let (cost_str, payment_amount) = match &cost_field {
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
    require_x402_payment_deferred(&state, &headers, payment_amount, "/api/sites", "Upload site")
        .await?;

    // Parse and validate zip
    let cursor = Cursor::new(&zip_bytes);
    let mut archive = ZipArchive::new(cursor).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid zip file: {}", e),
        )
            .into_response()
    })?;

    if archive.len() > MAX_FILES {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Too many files (max {})", MAX_FILES),
        )
            .into_response());
    }

    // Detect single-directory wrapper
    let prefix = detect_root_prefix(&mut archive);

    // Extract files and validate
    struct ExtractedFile {
        path: String,
        content_type: String,
        data: Vec<u8>,
    }

    let mut files: Vec<ExtractedFile> = Vec::new();
    let mut total_uncompressed: u64 = 0;
    let mut has_index = false;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to read zip entry: {}", e),
            )
                .into_response()
        })?;

        let raw_name = entry.name().to_string();

        if entry.is_dir() {
            continue;
        }

        // Strip the common prefix if detected
        let file_path = if let Some(ref pfx) = prefix {
            match raw_name.strip_prefix(pfx.as_str()) {
                Some(stripped) => stripped.to_string(),
                None => raw_name.clone(),
            }
        } else {
            raw_name.clone()
        };

        if file_path.is_empty() {
            continue;
        }

        // Security checks
        if file_path.contains("..") {
            return Err(
                (StatusCode::BAD_REQUEST, "Path traversal detected").into_response()
            );
        }
        if file_path.starts_with('.') || file_path.contains("/.") {
            continue; // Skip hidden files
        }
        if !utils::is_allowed_extension(&file_path) {
            continue; // Skip disallowed file types
        }

        if entry.size() > MAX_FILE_SIZE {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("File {} exceeds max size (5 MB)", file_path),
            )
                .into_response());
        }
        total_uncompressed += entry.size();
        if total_uncompressed > MAX_UNCOMPRESSED_SIZE {
            return Err((
                StatusCode::BAD_REQUEST,
                "Total uncompressed size exceeds 25 MB",
            )
                .into_response());
        }

        let mut data = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut data).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Failed to read file {}: {}", file_path, e),
            )
                .into_response()
        })?;

        if file_path == "index.html" {
            has_index = true;
        }

        let content_type = utils::content_type_for_path(&file_path).to_string();
        files.push(ExtractedFile {
            path: file_path,
            content_type,
            data,
        });
    }

    if !has_index {
        return Err(
            (StatusCode::BAD_REQUEST, "Zip must contain index.html at root").into_response()
        );
    }

    // If update: delete old file records (cascade doesn't apply here since we reuse the site row)
    if is_update {
        let site = existing_site.as_ref().unwrap();
        if let Err(e) = SiteService::delete_site_files(&state.pool, site.id).await {
            tracing::error!("Failed to delete old file records: {}", e);
        }
    }

    // Create or reuse site record
    let site = if is_update {
        existing_site.unwrap()
    } else {
        SiteService::create(
            &state.pool,
            auth.id,
            &slug,
            &title,
            description.as_deref(),
            Some(&cost_str),
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to create site: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create site").into_response()
        })?
    };

    // Insert files into Postgres
    let file_count = files.len();
    let mut total_size: i64 = 0;

    for file in &files {
        let size = file.data.len() as i64;
        total_size += size;

        SiteService::insert_file(
            &state.pool,
            site.id,
            &file.path,
            &file.content_type,
            size,
            &file.data,
        )
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert file record: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to store files").into_response()
        })?;
    }

    // Activate site
    SiteService::activate(&state.pool, site.id, file_count as i32, total_size)
        .await
        .map_err(|e| {
            tracing::error!("Failed to activate site: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to activate site").into_response()
        })?;

    // Record earnings
    if let Err(e) = EarningsService::record(&state.pool, "site", &cost_str, Some(auth.id)).await {
        tracing::error!("Failed to record site earnings: {}", e);
    }

    // Fetch updated site
    let site = SiteService::get_by_id(&state.pool, site.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get site: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        })?
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "Site not found").into_response())?;

    let url = format!("/s/{}", site.slug);
    Ok((
        StatusCode::CREATED,
        Json(SiteUploadResponse {
            site,
            url,
            files_uploaded: file_count,
        }),
    ))
}

async fn delete_site(
    State(state): State<AppState>,
    Path(slug): Path<String>,
    Extension(auth): Extension<AuthenticatedAgent>,
) -> Result<StatusCode, Response> {
    let site = SiteService::get_by_slug(&state.pool, &slug)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get site: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        })?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Site not found").into_response())?;

    if site.agent_id != auth.id {
        return Err((StatusCode::FORBIDDEN, "You don't own this site").into_response());
    }

    // Delete from DB (cascade deletes site_files)
    SiteService::delete(&state.pool, site.id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete site: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete site").into_response()
        })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Detect if all zip entries share a common top-level directory prefix.
fn detect_root_prefix<R: std::io::Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Option<String> {
    if archive.len() == 0 {
        return None;
    }

    let mut common_prefix: Option<String> = None;

    for i in 0..archive.len() {
        let entry = match archive.by_index_raw(i) {
            Ok(e) => e,
            Err(_) => return None,
        };
        let name = entry.name().to_string();

        let first_component = match name.split('/').next() {
            Some(c) if !c.is_empty() => c.to_string(),
            _ => return None,
        };

        match &common_prefix {
            None => common_prefix = Some(first_component),
            Some(existing) => {
                if *existing != first_component {
                    return None;
                }
            }
        }
    }

    common_prefix.map(|p| format!("{}/", p))
}
