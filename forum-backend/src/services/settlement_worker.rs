//! Background worker for processing settlement queue

use super::settlement_queue::{SettlementQueue, StoredVerifyRequest};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

/// Maximum retry attempts for a single settlement
const MAX_RETRIES: i32 = 5;

/// Background worker that processes settlements from the queue
pub struct SettlementWorker {
    queue: Arc<SettlementQueue>,
    facilitator_url: String,
    http_client: reqwest::Client,
}

impl SettlementWorker {
    pub fn new(
        queue: Arc<SettlementQueue>,
        facilitator_url: String,
        http_client: reqwest::Client,
    ) -> Self {
        Self {
            queue,
            facilitator_url,
            http_client,
        }
    }

    /// Run the worker until shutdown signal
    pub async fn run(&self, mut shutdown: broadcast::Receiver<()>) {
        info!("Settlement worker started");

        loop {
            // Try to claim next pending settlement
            let settlement = match self.queue.claim_next().await {
                Ok(Some(s)) => Some(s),
                Ok(None) => None,
                Err(e) => {
                    error!("Failed to claim settlement: {}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            if let Some(s) = settlement {
                let id = s.id;
                let nonce = s.nonce.clone();

                tokio::select! {
                    biased;

                    _ = shutdown.recv() => {
                        // Put it back
                        if let Err(e) = self.queue.record_retry(id, "Worker shutdown").await {
                            error!("Failed to re-queue settlement on shutdown: {}", e);
                        }
                        info!("Settlement worker shutting down");
                        break;
                    }

                    _ = self.process_settlement(s) => {
                        debug!("Processed settlement {}", nonce);
                    }
                }
            } else {
                // No pending, wait for notification or timeout
                tokio::select! {
                    biased;

                    _ = shutdown.recv() => {
                        info!("Settlement worker received shutdown");
                        break;
                    }

                    _ = self.queue.wait_for_items() => {}

                    _ = tokio::time::sleep(Duration::from_secs(5)) => {}
                }
            }
        }

        info!("Settlement worker stopped");
    }

    async fn process_settlement(&self, settlement: super::settlement_queue::StoredSettlement) {
        let id = settlement.id;
        let nonce = &settlement.nonce;

        // Parse the stored request
        let verify_request: StoredVerifyRequest = match serde_json::from_str(&settlement.verify_request_json) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to parse verify request for {}: {}", id, e);
                let _ = self.queue.mark_failed(id, &format!("Parse error: {}", e)).await;
                return;
            }
        };

        // Build settle request (same format as verify)
        let settle_url = format!("{}/settle", self.facilitator_url);

        let mut attempts = settlement.retry_count;
        let mut backoff = Duration::from_secs(2);

        loop {
            attempts += 1;

            let result = self
                .http_client
                .post(&settle_url)
                .json(&serde_json::json!({
                    "x402Version": verify_request.x402_version,
                    "paymentPayload": verify_request.payment_payload,
                    "paymentRequirements": verify_request.payment_requirements,
                }))
                .timeout(Duration::from_secs(60))
                .send()
                .await;

            match result {
                Ok(response) => {
                    if response.status().is_success() {
                        match response.json::<serde_json::Value>().await {
                            Ok(json) => {
                                let success = json.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
                                if success {
                                    let tx_hash = json
                                        .get("transaction")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown");
                                    info!("Settlement succeeded for nonce {}: tx {}", nonce, tx_hash);
                                    let _ = self.queue.mark_completed(id, tx_hash).await;
                                    return;
                                } else {
                                    let error = json
                                        .get("errorReason")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("Unknown error");

                                    if attempts >= MAX_RETRIES {
                                        error!("Settlement failed for nonce {} after {} attempts: {}", nonce, attempts, error);
                                        let _ = self.queue.mark_failed(id, error).await;
                                        return;
                                    }

                                    warn!("Settlement attempt {} failed for nonce {}: {}", attempts, nonce, error);
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse settle response: {}", e);
                            }
                        }
                    } else {
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        warn!("Settlement HTTP error for nonce {}: {} - {}", nonce, status, body);
                    }
                }
                Err(e) => {
                    warn!("Settlement request failed for nonce {}: {}", nonce, e);
                }
            }

            // Retry with backoff
            if attempts >= MAX_RETRIES {
                error!("Settlement exhausted retries for nonce {}", nonce);
                let _ = self.queue.mark_failed(id, "Max retries exceeded").await;
                return;
            }

            tokio::time::sleep(backoff).await;
            backoff = std::cmp::min(backoff * 2, Duration::from_secs(60));
        }
    }
}
