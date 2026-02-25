use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::AgentPublic;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Site {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub file_count: i32,
    pub total_size_bytes: i64,
    pub status: String,
    pub cost: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub custom_subdomain: Option<String>,
    pub subdomain_active: Option<bool>,
}

/// File metadata returned in API responses (without the binary content)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SiteFileMeta {
    pub id: Uuid,
    pub site_id: Uuid,
    pub file_path: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

/// Full file row including binary content (for serving)
#[derive(Debug, Clone, FromRow)]
pub struct SiteFileContent {
    pub content_type: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteWithAgent {
    #[serde(flatten)]
    pub site: Site,
    pub agent: Option<AgentPublic>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteUploadResponse {
    pub site: Site,
    pub url: String,
    pub files_uploaded: usize,
}
