use primitive_types::U256;
use sqlx::PgPool;
use uuid::Uuid;

pub struct EarningsService;

#[derive(Debug)]
pub struct EarningsBreakdown {
    /// Total earnings as raw token value string (256-bit, 18 decimals)
    pub total: String,
    /// Registration earnings as raw token value string
    pub registration_total: String,
    /// Post earnings as raw token value string
    pub post_total: String,
    pub registration_count: i64,
    pub post_count: i64,
}

impl EarningsService {
    /// Record an earning event with raw token amount string
    pub async fn record(
        pool: &PgPool,
        source: &str,
        amount: &str,
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

    /// Sum string amounts using U256 arithmetic
    fn sum_amounts(amounts: &[String]) -> String {
        let mut total = U256::zero();
        for amt in amounts {
            if let Ok(val) = U256::from_dec_str(amt) {
                total = total.saturating_add(val);
            }
        }
        total.to_string()
    }

    /// Get total earnings as raw token value string
    pub async fn get_total(pool: &PgPool) -> Result<String, sqlx::Error> {
        let amounts: Vec<(String,)> = sqlx::query_as(
            r#"SELECT amount FROM earnings"#
        )
        .fetch_all(pool)
        .await?;

        Ok(Self::sum_amounts(&amounts.into_iter().map(|(a,)| a).collect::<Vec<_>>()))
    }

    /// Get earnings breakdown by source
    pub async fn get_breakdown(pool: &PgPool) -> Result<EarningsBreakdown, sqlx::Error> {
        // Get all amounts grouped by source
        let registration_amounts: Vec<(String,)> = sqlx::query_as(
            r#"SELECT amount FROM earnings WHERE source = 'registration'"#
        )
        .fetch_all(pool)
        .await?;

        let post_amounts: Vec<(String,)> = sqlx::query_as(
            r#"SELECT amount FROM earnings WHERE source = 'post'"#
        )
        .fetch_all(pool)
        .await?;

        let registration_count = registration_amounts.len() as i64;
        let post_count = post_amounts.len() as i64;

        let registration_total = Self::sum_amounts(
            &registration_amounts.into_iter().map(|(a,)| a).collect::<Vec<_>>()
        );
        let post_total = Self::sum_amounts(
            &post_amounts.into_iter().map(|(a,)| a).collect::<Vec<_>>()
        );

        // Calculate total
        let reg_u256 = U256::from_dec_str(&registration_total).unwrap_or_default();
        let post_u256 = U256::from_dec_str(&post_total).unwrap_or_default();
        let total = reg_u256.saturating_add(post_u256).to_string();

        Ok(EarningsBreakdown {
            total,
            registration_total,
            post_total,
            registration_count,
            post_count,
        })
    }
}
