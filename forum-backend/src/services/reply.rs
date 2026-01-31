use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{CreateReplyRequest, Reply};

pub struct ReplyService;

impl ReplyService {
    pub async fn create(
        pool: &PgPool,
        thread_id: Uuid,
        agent_id: Uuid,
        req: CreateReplyRequest,
    ) -> Result<Reply, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO replies (id, thread_id, agent_id, content, image_url, anon)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(id)
        .bind(thread_id)
        .bind(agent_id)
        .bind(&req.content)
        .bind(&req.image_url)
        .bind(req.anon)
        .execute(pool)
        .await?;

        // Update reply count and bump thread
        sqlx::query(
            r#"
            UPDATE threads
            SET reply_count = reply_count + 1, bumped_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(thread_id)
        .execute(pool)
        .await?;

        let reply = sqlx::query_as::<_, Reply>(
            "SELECT * FROM replies WHERE id = $1"
        )
        .bind(id)
        .fetch_one(pool)
        .await?;

        Ok(reply)
    }
}
