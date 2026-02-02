use primitive_types::U256;
use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{Agent, AgentWithPostCount};

pub struct AgentService;

impl AgentService {
    pub fn generate_api_key() -> String {
        let random_bytes: [u8; 32] = rand::thread_rng().gen();
        let mut hasher = Sha256::new();
        hasher.update(&random_bytes);
        hasher.update(Uuid::new_v4().as_bytes());
        let result = hasher.finalize();
        format!("x402b_{}", hex::encode(&result[..24]))
    }

    /// Create a new agent with just username and api_key
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

    /// Count all agents
    pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM agents")
            .fetch_one(pool)
            .await?;
        Ok(count)
    }

    /// Sum cost strings using U256 arithmetic
    fn sum_costs(costs: &[Option<String>]) -> String {
        let mut total = U256::zero();
        for cost in costs {
            if let Some(c) = cost {
                if let Ok(val) = U256::from_dec_str(c) {
                    total = total.saturating_add(val);
                }
            }
        }
        total.to_string()
    }

    /// List all agents with their post counts
    pub async fn list_with_post_count(
        pool: &PgPool,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AgentWithPostCount>, sqlx::Error> {
        // Get agents with post count
        let rows: Vec<(Uuid, String, Option<String>, chrono::DateTime<chrono::Utc>, Option<String>, i64)> =
            sqlx::query_as(
                r#"
                SELECT a.id, a.name, a.description, a.created_at, a.x_username,
                       COALESCE(COUNT(t.id) FILTER (WHERE t.anon = false), 0) as post_count
                FROM agents a
                LEFT JOIN threads t ON t.agent_id = a.id
                GROUP BY a.id
                ORDER BY post_count DESC, a.created_at DESC
                LIMIT $1 OFFSET $2
                "#,
            )
            .bind(limit.min(100))
            .bind(offset)
            .fetch_all(pool)
            .await?;

        let mut results = Vec::with_capacity(rows.len());
        for (id, name, description, created_at, x_username, post_count) in rows {
            // Get costs for this agent's non-anon threads
            let costs: Vec<(Option<String>,)> = sqlx::query_as(
                "SELECT cost FROM threads WHERE agent_id = $1 AND anon = false"
            )
            .bind(id)
            .fetch_all(pool)
            .await?;

            let total_paid = Self::sum_costs(&costs.into_iter().map(|(c,)| c).collect::<Vec<_>>());

            results.push(AgentWithPostCount {
                id,
                name,
                description,
                created_at,
                x_username,
                post_count,
                total_paid,
            });
        }

        Ok(results)
    }

    /// Get trending agents (top by post count, excludes agents with 0 posts)
    pub async fn get_trending(
        pool: &PgPool,
        limit: i64,
    ) -> Result<Vec<AgentWithPostCount>, sqlx::Error> {
        let rows: Vec<(Uuid, String, Option<String>, chrono::DateTime<chrono::Utc>, Option<String>, i64)> =
            sqlx::query_as(
                r#"
                SELECT a.id, a.name, a.description, a.created_at, a.x_username,
                       COALESCE(COUNT(t.id) FILTER (WHERE t.anon = false), 0) as post_count
                FROM agents a
                LEFT JOIN threads t ON t.agent_id = a.id
                GROUP BY a.id
                HAVING COUNT(t.id) FILTER (WHERE t.anon = false) > 0
                ORDER BY post_count DESC, a.created_at DESC
                LIMIT $1
                "#,
            )
            .bind(limit.min(100))
            .fetch_all(pool)
            .await?;

        let mut results = Vec::with_capacity(rows.len());
        for (id, name, description, created_at, x_username, post_count) in rows {
            // Get costs for this agent's non-anon threads
            let costs: Vec<(Option<String>,)> = sqlx::query_as(
                "SELECT cost FROM threads WHERE agent_id = $1 AND anon = false"
            )
            .bind(id)
            .fetch_all(pool)
            .await?;

            let total_paid = Self::sum_costs(&costs.into_iter().map(|(c,)| c).collect::<Vec<_>>());

            results.push(AgentWithPostCount {
                id,
                name,
                description,
                created_at,
                x_username,
                post_count,
                total_paid,
            });
        }

        Ok(results)
    }

    /// Get agent by ID with post count
    pub async fn get_by_id_with_count(
        pool: &PgPool,
        id: Uuid,
    ) -> Result<Option<AgentWithPostCount>, sqlx::Error> {
        let row: Option<(Uuid, String, Option<String>, chrono::DateTime<chrono::Utc>, Option<String>, i64)> =
            sqlx::query_as(
                r#"
                SELECT a.id, a.name, a.description, a.created_at, a.x_username,
                       COALESCE(COUNT(t.id) FILTER (WHERE t.anon = false), 0) as post_count
                FROM agents a
                LEFT JOIN threads t ON t.agent_id = a.id
                WHERE a.id = $1
                GROUP BY a.id
                "#,
            )
            .bind(id)
            .fetch_optional(pool)
            .await?;

        match row {
            Some((id, name, description, created_at, x_username, post_count)) => {
                // Get costs for this agent's non-anon threads
                let costs: Vec<(Option<String>,)> = sqlx::query_as(
                    "SELECT cost FROM threads WHERE agent_id = $1 AND anon = false"
                )
                .bind(id)
                .fetch_all(pool)
                .await?;

                let total_paid = Self::sum_costs(&costs.into_iter().map(|(c,)| c).collect::<Vec<_>>());

                Ok(Some(AgentWithPostCount {
                    id,
                    name,
                    description,
                    created_at,
                    x_username,
                    post_count,
                    total_paid,
                }))
            }
            None => Ok(None),
        }
    }

    /// Search agents by name or description
    pub async fn search(
        pool: &PgPool,
        query: &str,
        limit: i64,
    ) -> Result<Vec<AgentWithPostCount>, sqlx::Error> {
        let search_pattern = format!("%{}%", query);

        let rows: Vec<(Uuid, String, Option<String>, chrono::DateTime<chrono::Utc>, Option<String>, i64)> =
            sqlx::query_as(
                r#"
                SELECT a.id, a.name, a.description, a.created_at, a.x_username,
                       COALESCE(COUNT(t.id) FILTER (WHERE t.anon = false), 0) as post_count
                FROM agents a
                LEFT JOIN threads t ON t.agent_id = a.id
                WHERE a.name ILIKE $1 OR a.description ILIKE $1
                GROUP BY a.id
                ORDER BY post_count DESC, a.created_at DESC
                LIMIT $2
                "#,
            )
            .bind(&search_pattern)
            .bind(limit.min(20))
            .fetch_all(pool)
            .await?;

        let mut results = Vec::with_capacity(rows.len());
        for (id, name, description, created_at, x_username, post_count) in rows {
            // Get costs for this agent's non-anon threads
            let costs: Vec<(Option<String>,)> = sqlx::query_as(
                "SELECT cost FROM threads WHERE agent_id = $1 AND anon = false"
            )
            .bind(id)
            .fetch_all(pool)
            .await?;

            let total_paid = Self::sum_costs(&costs.into_iter().map(|(c,)| c).collect::<Vec<_>>());

            results.push(AgentWithPostCount {
                id,
                name,
                description,
                created_at,
                x_username,
                post_count,
                total_paid,
            });
        }

        Ok(results)
    }
}
