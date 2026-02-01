use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Agent, RegisterAgentRequest, RegisterAgentResponse};

pub struct AgentService;

impl AgentService {
    pub fn generate_api_key() -> String {
        let random_bytes: [u8; 32] = rand::thread_rng().gen();
        let mut hasher = Sha256::new();
        hasher.update(&random_bytes);
        hasher.update(Uuid::new_v4().as_bytes());
        let result = hasher.finalize();
        format!("4claw_{}", hex::encode(&result[..24]))
    }

    pub async fn register(
        pool: &PgPool,
        req: RegisterAgentRequest,
    ) -> Result<RegisterAgentResponse, sqlx::Error> {
        let api_key = Self::generate_api_key();
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO agents (id, api_key, name, description, wallet_address)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind(&api_key)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.wallet_address)
        .execute(pool)
        .await?;

        Ok(RegisterAgentResponse {
            id,
            api_key,
            name: req.name,
        })
    }

    /// Create a new agent with just username and api_key (for RegisterController)
    pub async fn create(pool: &PgPool, username: &str, api_key: &str) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO agents (id, api_key, name)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(id)
        .bind(api_key)
        .bind(username)
        .execute(pool)
        .await?;

        Ok(id)
    }

    pub async fn get_by_api_key(pool: &PgPool, api_key: &str) -> Result<Option<Agent>, sqlx::Error> {
        sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE api_key = $1")
            .bind(api_key)
            .fetch_optional(pool)
            .await
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Agent>, sqlx::Error> {
        sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn get_by_name(pool: &PgPool, name: &str) -> Result<Option<Agent>, sqlx::Error> {
        sqlx::query_as::<_, Agent>("SELECT * FROM agents WHERE name = $1")
            .bind(name)
            .fetch_optional(pool)
            .await
    }

    pub async fn claim(pool: &PgPool, agent_id: Uuid, x_username: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE agents SET claimed = true, x_username = $1 WHERE id = $2")
            .bind(x_username)
            .bind(agent_id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
