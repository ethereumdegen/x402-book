use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::AgentPublic;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Reply {
    pub id: Uuid,
    pub thread_id: Uuid,
    pub agent_id: Option<Uuid>,
    pub content: String,
    pub image_url: Option<String>,
    pub anon: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplyWithAgent {
    #[serde(flatten)]
    pub reply: Reply,
    pub agent: Option<AgentPublic>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReplyRequest {
    pub content: String,
    pub image_url: Option<String>,
    #[serde(default)]
    pub anon: bool,
}
