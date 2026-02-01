use sqlx::PgPool;
use uuid::Uuid;

pub struct EarningsService;

#[derive(Debug)]
pub struct EarningsBreakdown {
    pub total: i64,
    pub registration_total: i64,
    pub post_total: i64,
    pub registration_count: i64,
    pub post_count: i64,
}

impl EarningsService {
    /// Record an earning event
    pub async fn record(
        pool: &PgPool,
        source: &str,
        amount: i64,
        agent_id: Option<Uuid>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO earnings (source, amount, agent_id)
            VALUES ($1, $2, $3)
            "#,
            source,
            amount,
            agent_id
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Get total earnings
    pub async fn get_total(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT COALESCE(SUM(amount), 0)::BIGINT as "total!" FROM earnings"#
        )
        .fetch_one(pool)
        .await?;
        Ok(result)
    }

    /// Get earnings breakdown by source
    pub async fn get_breakdown(pool: &PgPool) -> Result<EarningsBreakdown, sqlx::Error> {
        let total = Self::get_total(pool).await?;

        let registration = sqlx::query!(
            r#"
            SELECT
                COALESCE(SUM(amount), 0)::BIGINT as "total!",
                COUNT(*) as "count!"
            FROM earnings
            WHERE source = 'registration'
            "#
        )
        .fetch_one(pool)
        .await?;

        let post = sqlx::query!(
            r#"
            SELECT
                COALESCE(SUM(amount), 0)::BIGINT as "total!",
                COUNT(*) as "count!"
            FROM earnings
            WHERE source = 'post'
            "#
        )
        .fetch_one(pool)
        .await?;

        Ok(EarningsBreakdown {
            total,
            registration_total: registration.total,
            post_total: post.total,
            registration_count: registration.count,
            post_count: post.count,
        })
    }
}
