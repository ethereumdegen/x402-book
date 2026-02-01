//! Settlement queue for decoupling x402 settlement from HTTP request flow
//!
//! This module provides a FIFO queue for pending settlements that allows
//! the HTTP request to return immediately after payment verification,
//! while settlement is processed asynchronously by a background worker.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Status of a settlement
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum SettlementStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl SettlementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SettlementStatus::Pending => "pending",
            SettlementStatus::InProgress => "in_progress",
            SettlementStatus::Completed => "completed",
            SettlementStatus::Failed => "failed",
        }
    }
}

/// A pending settlement stored in the database
#[derive(Debug, Clone)]
pub struct StoredSettlement {
    pub id: Uuid,
    pub nonce: String,
    pub verify_request_json: String,
    pub status: String,
    pub retry_count: i32,
    pub last_error: Option<String>,
    pub tx_hash: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// The verify request data we need to store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredVerifyRequest {
    pub x402_version: u32,
    pub payment_payload: serde_json::Value,
    pub payment_requirements: serde_json::Value,
}

/// FIFO queue for pending settlements backed by Postgres
pub struct SettlementQueue {
    pool: PgPool,
    notify: Arc<Notify>,
    len: AtomicUsize,
}

impl SettlementQueue {
    /// Create a new settlement queue
    pub async fn new(pool: PgPool) -> Result<Self, sqlx::Error> {
        // Create table if not exists
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS settlements (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                nonce TEXT UNIQUE NOT NULL,
                verify_request_json TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                retry_count INTEGER NOT NULL DEFAULT 0,
                last_error TEXT,
                tx_hash TEXT,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // Create index on status
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_settlements_status ON settlements(status)",
        )
        .execute(&pool)
        .await?;

        // Recover any in_progress settlements from previous run
        let recovered = sqlx::query(
            "UPDATE settlements SET status = 'pending', updated_at = NOW() WHERE status = 'in_progress'",
        )
        .execute(&pool)
        .await?
        .rows_affected();

        if recovered > 0 {
            info!("Recovered {} in-progress settlements from previous session", recovered);
        }

        // Get initial pending count
        let pending: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM settlements WHERE status = 'pending'")
            .fetch_one(&pool)
            .await?;

        if pending.0 > 0 {
            info!("Loaded {} pending settlements from database", pending.0);
        }

        Ok(Self {
            pool,
            notify: Arc::new(Notify::new()),
            len: AtomicUsize::new(pending.0 as usize),
        })
    }

    /// Push a settlement to the queue
    pub async fn push(&self, nonce: &str, verify_request: &StoredVerifyRequest) -> Result<bool, sqlx::Error> {
        let json = serde_json::to_string(verify_request).unwrap();

        let result = sqlx::query(
            r#"
            INSERT INTO settlements (nonce, verify_request_json, status)
            VALUES ($1, $2, 'pending')
            ON CONFLICT (nonce) DO NOTHING
            "#,
        )
        .bind(nonce)
        .bind(&json)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() > 0 {
            self.len.fetch_add(1, Ordering::SeqCst);
            debug!("Queued settlement for nonce {}", nonce);
            self.notify.notify_one();
            Ok(true)
        } else {
            debug!("Settlement for nonce {} already exists", nonce);
            Ok(false)
        }
    }

    /// Claim the next pending settlement (FIFO)
    pub async fn claim_next(&self) -> Result<Option<StoredSettlement>, sqlx::Error> {
        let result: Option<StoredSettlement> = sqlx::query_as::<_, (Uuid, String, String, String, i32, Option<String>, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"
            UPDATE settlements
            SET status = 'in_progress', updated_at = NOW()
            WHERE id = (
                SELECT id FROM settlements
                WHERE status = 'pending'
                ORDER BY created_at ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING id, nonce, verify_request_json, status, retry_count, last_error, tx_hash, created_at, updated_at
            "#,
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|(id, nonce, verify_request_json, status, retry_count, last_error, tx_hash, created_at, updated_at)| {
            StoredSettlement {
                id,
                nonce,
                verify_request_json,
                status,
                retry_count,
                last_error,
                tx_hash,
                created_at,
                updated_at,
            }
        });

        if result.is_some() {
            self.len.fetch_sub(1, Ordering::SeqCst);
        }

        Ok(result)
    }

    /// Mark settlement as completed
    pub async fn mark_completed(&self, id: Uuid, tx_hash: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE settlements SET status = 'completed', tx_hash = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(tx_hash)
        .bind(id)
        .execute(&self.pool)
        .await?;

        debug!("Marked settlement {} as completed, tx: {}", id, tx_hash);
        Ok(())
    }

    /// Mark settlement as failed
    pub async fn mark_failed(&self, id: Uuid, error: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE settlements SET status = 'failed', last_error = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await?;

        warn!("Marked settlement {} as failed: {}", id, error);
        Ok(())
    }

    /// Record a retry attempt
    pub async fn record_retry(&self, id: Uuid, error: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE settlements SET status = 'pending', retry_count = retry_count + 1, last_error = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await?;

        self.len.fetch_add(1, Ordering::SeqCst);
        debug!("Recorded retry for settlement {}", id);
        Ok(())
    }

    /// Get current queue length
    pub fn len(&self) -> usize {
        self.len.load(Ordering::SeqCst)
    }

    /// Wait for new items
    pub async fn wait_for_items(&self) {
        self.notify.notified().await;
    }

    /// Notify all waiters (for shutdown)
    pub fn notify_all(&self) {
        self.notify.notify_waiters();
    }

    /// Get the database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
