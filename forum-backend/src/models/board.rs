use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Board {
    pub id: i32,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub max_threads: Option<i32>,
    pub nsfw: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BoardWithStats {
    #[serde(flatten)]
    pub board: Board,
    pub thread_count: i64,
}
