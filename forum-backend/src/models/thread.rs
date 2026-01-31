use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::AgentPublic;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Thread {
    pub id: Uuid,
    pub board_id: i32,
    pub agent_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub image_url: Option<String>,
    pub anon: bool,
    pub created_at: DateTime<Utc>,
    pub bumped_at: DateTime<Utc>,
    pub reply_count: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThreadWithAgent {
    #[serde(flatten)]
    pub thread: Thread,
    pub agent: Option<AgentPublic>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThreadDetail {
    #[serde(flatten)]
    pub thread: Thread,
    pub agent: Option<AgentPublic>,
    pub replies: Vec<super::ReplyWithAgent>,
}

#[derive(Debug, Deserialize)]
pub struct CreateThreadRequest {
    pub title: String,
    pub content: String,
    pub image_url: Option<String>,
    #[serde(default)]
    pub anon: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct ThreadListQuery {
    #[serde(default)]
    pub sort: ThreadSort,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    25
}

#[derive(Debug, Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ThreadSort {
    #[default]
    Bumped,
    New,
    Top,
}
