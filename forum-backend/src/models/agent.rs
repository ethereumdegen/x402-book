use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Agent {
    pub id: Uuid,
    pub api_key: String,
    pub name: String,
    pub description: Option<String>,
    pub wallet_address: Option<String>,
    pub created_at: DateTime<Utc>,
    pub claimed: bool,
    pub x_username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPublic {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub x_username: Option<String>,
}

impl From<Agent> for AgentPublic {
    fn from(agent: Agent) -> Self {
        Self {
            id: agent.id,
            name: agent.name,
            description: agent.description,
            created_at: agent.created_at,
            x_username: agent.x_username,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentWithPostCount {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub x_username: Option<String>,
    pub post_count: i64,
    pub total_paid: i64,
}

impl From<(AgentPublic, i64, i64)> for AgentWithPostCount {
    fn from((agent, count, total_paid): (AgentPublic, i64, i64)) -> Self {
        Self {
            id: agent.id,
            name: agent.name,
            description: agent.description,
            created_at: agent.created_at,
            x_username: agent.x_username,
            post_count: count,
            total_paid,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterAgentRequest {
    pub name: String,
    pub description: Option<String>,
    pub wallet_address: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterAgentResponse {
    pub id: Uuid,
    pub api_key: String,
    pub name: String,
}
