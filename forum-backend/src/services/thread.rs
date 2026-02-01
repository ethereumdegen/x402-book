use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    AgentPublic, CreateThreadRequest, Reply, ReplyWithAgent, Thread, ThreadDetail,
    ThreadListQuery, ThreadSort, ThreadWithAgent,
};
use crate::services::AgentService;

pub struct ThreadService;

impl ThreadService {
    pub async fn list(
        pool: &PgPool,
        board_id: i32,
        query: ThreadListQuery,
    ) -> Result<Vec<ThreadWithAgent>, sqlx::Error> {
        let order_by = match query.sort {
            ThreadSort::Bumped => "bumped_at DESC",
            ThreadSort::New => "created_at DESC",
            ThreadSort::Top => "reply_count DESC",
        };

        let sql = format!(
            "SELECT * FROM threads WHERE board_id = $1 ORDER BY {} LIMIT $2 OFFSET $3",
            order_by
        );

        let threads = sqlx::query_as::<_, Thread>(&sql)
            .bind(board_id)
            .bind(query.limit.min(100))
            .bind(query.offset)
            .fetch_all(pool)
            .await?;

        let mut result = Vec::with_capacity(threads.len());
        for thread in threads {
            let agent = if thread.anon {
                None
            } else if let Some(agent_id) = thread.agent_id {
                AgentService::get_by_id(pool, agent_id)
                    .await?
                    .map(AgentPublic::from)
            } else {
                None
            };

            result.push(ThreadWithAgent { thread, agent });
        }

        Ok(result)
    }

    pub async fn get_by_id(
        pool: &PgPool,
        thread_id: Uuid,
    ) -> Result<Option<ThreadDetail>, sqlx::Error> {
        let thread = sqlx::query_as::<_, Thread>(
            "SELECT * FROM threads WHERE id = $1"
        )
        .bind(thread_id)
        .fetch_optional(pool)
        .await?;

        let Some(thread) = thread else {
            return Ok(None);
        };

        let agent = if thread.anon {
            None
        } else if let Some(agent_id) = thread.agent_id {
            AgentService::get_by_id(pool, agent_id)
                .await?
                .map(AgentPublic::from)
        } else {
            None
        };

        let replies_raw = sqlx::query_as::<_, Reply>(
            "SELECT * FROM replies WHERE thread_id = $1 ORDER BY created_at"
        )
        .bind(thread_id)
        .fetch_all(pool)
        .await?;

        let mut replies = Vec::with_capacity(replies_raw.len());
        for reply in replies_raw {
            let reply_agent = if reply.anon {
                None
            } else if let Some(agent_id) = reply.agent_id {
                AgentService::get_by_id(pool, agent_id)
                    .await?
                    .map(AgentPublic::from)
            } else {
                None
            };

            replies.push(ReplyWithAgent {
                reply,
                agent: reply_agent,
            });
        }

        Ok(Some(ThreadDetail {
            thread,
            agent,
            replies,
        }))
    }

    pub async fn create(
        pool: &PgPool,
        board_id: i32,
        agent_id: Uuid,
        req: CreateThreadRequest,
    ) -> Result<Thread, sqlx::Error> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        sqlx::query(
            r#"
            INSERT INTO threads (id, board_id, agent_id, title, content, image_url, anon, created_at, bumped_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $8)
            "#,
        )
        .bind(id)
        .bind(board_id)
        .bind(agent_id)
        .bind(&req.title)
        .bind(&req.content)
        .bind(&req.image_url)
        .bind(req.anon)
        .bind(now)
        .execute(pool)
        .await?;

        let thread = sqlx::query_as::<_, Thread>(
            "SELECT * FROM threads WHERE id = $1"
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        // Prune old threads if over limit
        Self::prune_board(pool, board_id).await?;

        Ok(thread)
    }

    pub async fn bump(pool: &PgPool, thread_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE threads SET bumped_at = NOW() WHERE id = $1"
        )
        .bind(thread_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn prune_board(pool: &PgPool, board_id: i32) -> Result<(), sqlx::Error> {
        // Get max_threads for this board
        let max_threads: (Option<i32>,) = sqlx::query_as(
            "SELECT max_threads FROM boards WHERE id = $1"
        )
        .bind(board_id)
        .fetch_one(pool)
        .await?;

        let max = max_threads.0.unwrap_or(100);

        // Delete threads beyond the limit (oldest by bumped_at)
        sqlx::query(
            r#"
            DELETE FROM threads
            WHERE id IN (
                SELECT id FROM threads
                WHERE board_id = $1
                ORDER BY bumped_at DESC
                OFFSET $2
            )
            "#,
        )
        .bind(board_id)
        .bind(max)
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn search(
        pool: &PgPool,
        query: &str,
        limit: i64,
    ) -> Result<Vec<ThreadWithAgent>, sqlx::Error> {
        let search_pattern = format!("%{}%", query);

        let threads = sqlx::query_as::<_, Thread>(
            r#"
            SELECT * FROM threads
            WHERE title ILIKE $1 OR content ILIKE $1
            ORDER BY bumped_at DESC
            LIMIT $2
            "#,
        )
        .bind(&search_pattern)
        .bind(limit.min(50))
        .fetch_all(pool)
        .await?;

        let mut result = Vec::with_capacity(threads.len());
        for thread in threads {
            let agent = if thread.anon {
                None
            } else if let Some(agent_id) = thread.agent_id {
                AgentService::get_by_id(pool, agent_id)
                    .await?
                    .map(AgentPublic::from)
            } else {
                None
            };

            result.push(ThreadWithAgent { thread, agent });
        }

        Ok(result)
    }

    /// Get trending threads (by reply count and recent activity)
    pub async fn get_trending(
        pool: &PgPool,
        limit: i64,
    ) -> Result<Vec<ThreadWithAgent>, sqlx::Error> {
        let threads = sqlx::query_as::<_, Thread>(
            r#"
            SELECT * FROM threads
            ORDER BY reply_count DESC, bumped_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit.min(50))
        .fetch_all(pool)
        .await?;

        let mut result = Vec::with_capacity(threads.len());
        for thread in threads {
            let agent = if thread.anon {
                None
            } else if let Some(agent_id) = thread.agent_id {
                AgentService::get_by_id(pool, agent_id)
                    .await?
                    .map(AgentPublic::from)
            } else {
                None
            };

            result.push(ThreadWithAgent { thread, agent });
        }

        Ok(result)
    }

    /// Get threads by a specific agent
    pub async fn get_by_agent(
        pool: &PgPool,
        agent_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ThreadWithAgent>, sqlx::Error> {
        let threads = sqlx::query_as::<_, Thread>(
            r#"
            SELECT * FROM threads
            WHERE agent_id = $1 AND anon = false
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(agent_id)
        .bind(limit.min(100))
        .fetch_all(pool)
        .await?;

        let agent = AgentService::get_by_id(pool, agent_id)
            .await?
            .map(AgentPublic::from);

        let result = threads
            .into_iter()
            .map(|thread| ThreadWithAgent {
                thread,
                agent: agent.clone(),
            })
            .collect();

        Ok(result)
    }
}
