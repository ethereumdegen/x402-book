use axum::{
    body::Body,
    http::{header, HeaderMap, StatusCode},
    response::Response,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

use crate::models::x402::{
    PaymentRequiredResponse, PaymentRequirements, SettleResponse, VerifyRequest, VerifyResponse,
};
use crate::AppState;

use crate::config::Config;
use crate::domain_types::DomainU256;

/// Build payment requirements from config
fn build_payment_requirements(
    config: &Config,
    amount: DomainU256,
    resource: &str,
    description: &str,
) -> PaymentRequirements {
    PaymentRequirements {
        scheme: "permit".to_string(),
        network: config.payment_network.clone(),
        max_amount_required: amount.to_string(),
        resource: resource.to_string(),
        description: description.to_string(),
        mime_type: "application/json".to_string(),
        pay_to: config.wallet_address.clone(),
        max_timeout_seconds: 300, // 5 minutes
        asset: config.payment_token_address.clone(),
        extra: Some(serde_json::json!({
            "token": config.payment_token_symbol,
            "address": config.payment_token_address,
            "decimals": config.payment_token_decimals,
            "name": config.payment_token_name,
            "version": config.payment_token_version,
            "facilitatorSigner": config.facilitator_signer
        })),
    }
}

/// Generate a 402 Payment Required response
pub fn payment_required_response(
    config: &Config,
    amount: DomainU256,
    resource: &str,
    description: &str,
) -> Response {
    let requirements = build_payment_requirements(config, amount, resource, description);

    let response = PaymentRequiredResponse {
        x402_version: 1,
        accepts: vec![requirements],
        error: None,
    };

    let body = serde_json::to_string(&response).unwrap_or_default();

    Response::builder()
        .status(StatusCode::PAYMENT_REQUIRED)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .unwrap()
}

/// Decode payment header and build verify request
fn build_verify_request(
    payment_header: &str,
    payment_requirements: PaymentRequirements,
) -> Result<VerifyRequest, String> {
    let payload_bytes = BASE64
        .decode(payment_header)
        .map_err(|e| format!("Invalid payment header encoding: {}", e))?;

    let payment_payload: serde_json::Value = serde_json::from_slice(&payload_bytes)
        .map_err(|e| format!("Invalid payment payload JSON: {}", e))?;

    Ok(VerifyRequest {
        x402_version: 1,
        payment_payload,
        payment_requirements,
    })
}

/// Verify payment with facilitator
async fn verify_payment(
    http_client: &reqwest::Client,
    facilitator_url: &str,
    verify_request: &VerifyRequest,
) -> Result<VerifyResponse, String> {
    let verify_url = format!("{}/verify", facilitator_url);

    let response = http_client
        .post(&verify_url)
        .json(verify_request)
        .send()
        .await
        .map_err(|e| format!("Failed to contact facilitator: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Facilitator returned error: {} - {}", status, body));
    }

    response
        .json::<VerifyResponse>()
        .await
        .map_err(|e| format!("Failed to parse verify response: {}", e))
}

/// Settle payment with facilitator (uses same request format as verify)
async fn settle_payment(
    http_client: &reqwest::Client,
    facilitator_url: &str,
    settle_request: &VerifyRequest,
) -> Result<SettleResponse, String> {
    let settle_url = format!("{}/settle", facilitator_url);

    let response = http_client
        .post(&settle_url)
        .json(settle_request)
        .send()
        .await
        .map_err(|e| format!("Failed to contact facilitator for settlement: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Facilitator settlement error: {} - {}", status, body));
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
    amount: DomainU256,
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
            // Build payment requirements (must match what we return in 402)
            let payment_requirements =
                build_payment_requirements(&state.config, amount, resource, description);

            // Build verify request
            let verify_request = build_verify_request(payment, payment_requirements)
                .map_err(|e| {
                    tracing::error!("Failed to build verify request: {}", e);
                    payment_error_response(StatusCode::BAD_REQUEST, &e)
                })?;

            // Verify payment
            match verify_payment(
                &state.http_client,
                &state.config.facilitator_url,
                &verify_request,
            )
            .await
            {
                Ok(verify_response) => {
                    if verify_response.is_valid {
                        // Settle the payment using the same request
                        match settle_payment(
                            &state.http_client,
                            &state.config.facilitator_url,
                            &verify_request,
                        )
                        .await
                        {
                            Ok(settle_response) => {
                                if settle_response.success {
                                    tracing::info!(
                                        "Payment settled: {:?}",
                                        settle_response.transaction
                                    );
                                    Ok(settle_response.transaction)
                                } else {
                                    tracing::error!(
                                        "Settlement failed: {:?}",
                                        settle_response.error_reason
                                    );
                                    Err(payment_error_response(
                                        StatusCode::PAYMENT_REQUIRED,
                                        &format!(
                                            "Payment settlement failed: {}",
                                            settle_response.error_reason.unwrap_or_default()
                                        ),
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
                        tracing::warn!(
                            "Payment verification failed: {:?}",
                            verify_response.invalid_reason
                        );
                        Err(payment_error_response(
                            StatusCode::PAYMENT_REQUIRED,
                            &format!(
                                "Payment verification failed: {}",
                                verify_response.invalid_reason.unwrap_or_default()
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
