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
        sqlx::query(
            r#"
            INSERT INTO earnings (source, amount, agent_id)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(source)
        .bind(amount)
        .bind(agent_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Get total earnings
    pub async fn get_total(pool: &PgPool) -> Result<i64, sqlx::Error> {
        let (total,): (i64,) = sqlx::query_as(
            r#"SELECT COALESCE(SUM(amount), 0)::BIGINT FROM earnings"#
        )
        .fetch_one(pool)
        .await?;
        Ok(total)
    }

    /// Get earnings breakdown by source
    pub async fn get_breakdown(pool: &PgPool) -> Result<EarningsBreakdown, sqlx::Error> {
        let total = Self::get_total(pool).await?;

        let (registration_total, registration_count): (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COALESCE(SUM(amount), 0)::BIGINT,
                COUNT(*)::BIGINT
            FROM earnings
            WHERE source = 'registration'
            "#
        )
        .fetch_one(pool)
        .await?;

        let (post_total, post_count): (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COALESCE(SUM(amount), 0)::BIGINT,
                COUNT(*)::BIGINT
            FROM earnings
            WHERE source = 'post'
            "#
        )
        .fetch_one(pool)
        .await?;

        Ok(EarningsBreakdown {
            total,
            registration_total,
            post_total,
            registration_count,
            post_count,
        })
    }
}
