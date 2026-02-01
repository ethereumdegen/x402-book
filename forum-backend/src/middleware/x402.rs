use axum::{
    body::Body,
    http::{header, HeaderMap, StatusCode},
    response::Response,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

use crate::models::x402::{
    PaymentOption, PaymentRequiredResponse, SettleRequest, SettleResponse, VerifyRequest,
    VerifyResponse,
};
use crate::AppState;

use crate::config::Config;

/// Generate a 402 Payment Required response
pub fn payment_required_response(
    config: &Config,
    amount: u64,
    resource: &str,
    description: &str,
) -> Response {
    let response = PaymentRequiredResponse {
        x402_version: 1,
        accepts: vec![PaymentOption {
            scheme: "exact".to_string(),
            network: config.payment_network.clone(),
            max_amount_required: amount.to_string(),
            resource: resource.to_string(),
            description: description.to_string(),
            pay_to: config.wallet_address.clone(),
            extra: Some(serde_json::json!({
                "token": config.payment_token_symbol,
                "address": config.payment_token_address,
                "decimals": config.payment_token_decimals
            })),
        }],
        error: None,
    };

    let body = serde_json::to_string(&response).unwrap_or_default();

    Response::builder()
        .status(StatusCode::PAYMENT_REQUIRED)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap()
}

/// Verify payment with facilitator
async fn verify_payment(
    http_client: &reqwest::Client,
    facilitator_url: &str,
    payment_header: &str,
) -> Result<VerifyResponse, String> {
    let payload_bytes = BASE64
        .decode(payment_header)
        .map_err(|e| format!("Invalid payment header encoding: {}", e))?;

    let payload: serde_json::Value = serde_json::from_slice(&payload_bytes)
        .map_err(|e| format!("Invalid payment payload JSON: {}", e))?;

    let verify_url = format!("{}/verify", facilitator_url);

    let response = http_client
        .post(&verify_url)
        .json(&VerifyRequest { payload })
        .send()
        .await
        .map_err(|e| format!("Failed to contact facilitator: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Facilitator returned error: {}", response.status()));
    }

    response
        .json::<VerifyResponse>()
        .await
        .map_err(|e| format!("Failed to parse verify response: {}", e))
}

/// Settle payment with facilitator
async fn settle_payment(
    http_client: &reqwest::Client,
    facilitator_url: &str,
    payment_id: &str,
) -> Result<SettleResponse, String> {
    let settle_url = format!("{}/settle", facilitator_url);

    let response = http_client
        .post(&settle_url)
        .json(&SettleRequest {
            payment_id: payment_id.to_string(),
        })
        .send()
        .await
        .map_err(|e| format!("Failed to contact facilitator for settlement: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Facilitator settlement error: {}",
            response.status()
        ));
    }

    response
        .json::<SettleResponse>()
        .await
        .map_err(|e| format!("Failed to parse settle response: {}", e))
}

/// Error response for payment failures
fn payment_error_response(status: StatusCode, message: &str) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(message.to_string()))
        .unwrap()
}

/// Require x402 payment - checks header, verifies, and settles
/// Returns Ok(Option<tx_hash>) on success, Err(Response) on failure
pub async fn require_x402_payment(
    state: &AppState,
    headers: &HeaderMap,
    amount: u64,
    resource: &str,
    description: &str,
) -> Result<Option<String>, Response> {
    let payment_header = headers.get("X-PAYMENT").and_then(|v| v.to_str().ok());

    match payment_header {
        None => {
            // No payment header, return 402
            Err(payment_required_response(
                &state.config,
                amount,
                resource,
                description,
            ))
        }
        Some(payment) => {
            // Verify payment
            match verify_payment(&state.http_client, &state.config.facilitator_url, payment).await {
                Ok(verify_response) => {
                    if verify_response.valid {
                        // Settle the payment
                        if let Some(payment_id) = verify_response.payment_id {
                            match settle_payment(
                                &state.http_client,
                                &state.config.facilitator_url,
                                &payment_id,
                            )
                            .await
                            {
                                Ok(settle_response) => {
                                    if settle_response.settled {
                                        tracing::info!(
                                            "Payment settled: {:?}",
                                            settle_response.tx_hash
                                        );
                                        Ok(settle_response.tx_hash)
                                    } else {
                                        tracing::error!(
                                            "Settlement failed: {:?}",
                                            settle_response.error
                                        );
                                        Err(payment_error_response(
                                            StatusCode::PAYMENT_REQUIRED,
                                            "Payment settlement failed",
                                        ))
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Settlement error: {}", e);
                                    Err(payment_error_response(
                                        StatusCode::BAD_GATEWAY,
                                        &format!("Settlement error: {}", e),
                                    ))
                                }
                            }
                        } else {
                            tracing::info!("Payment verified (no payment_id returned)");
                            Ok(None)
                        }
                    } else {
                        tracing::warn!(
                            "Payment verification failed: {:?}",
                            verify_response.error
                        );
                        Err(payment_error_response(
                            StatusCode::PAYMENT_REQUIRED,
                            &format!(
                                "Payment verification failed: {}",
                                verify_response.error.unwrap_or_default()
                            ),
                        ))
                    }
                }
                Err(e) => {
                    tracing::error!("Payment verification error: {}", e);
                    Err(payment_error_response(
                        StatusCode::BAD_GATEWAY,
                        &format!("Payment verification error: {}", e),
                    ))
                }
            }
        }
    }
}
