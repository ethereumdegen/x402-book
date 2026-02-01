use std::env;

use crate::domain_types::DomainU256;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub facilitator_url: String,
    pub wallet_address: String,
    pub cost_per_registration: DomainU256,
    pub cost_per_post: DomainU256,
    // Payment token configuration
    pub payment_network: String,
    pub payment_token_address: String,
    pub payment_token_symbol: String,
    pub payment_token_decimals: u8,
    // EIP-712 domain info for signing
    pub payment_token_name: String,
    pub payment_token_version: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            port: env::var("BACKEND_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("BACKEND_PORT must be a valid port number"),
            facilitator_url: env::var("FACILITATOR_URL")
                .unwrap_or_else(|_| "https://facilitator.x402.org".to_string()),
            wallet_address: env::var("WALLET_ADDRESS").expect("WALLET_ADDRESS must be set"),
            cost_per_registration: DomainU256::from_string(
                &env::var("COST_PER_REGISTRATION").unwrap_or_else(|_| "5000".to_string()),
            )
            .expect("COST_PER_REGISTRATION must be a valid U256"),
            cost_per_post: DomainU256::from_string(
                &env::var("COST_PER_POST").unwrap_or_else(|_| "1000".to_string()),
            )
            .expect("COST_PER_POST must be a valid U256"),
            // Payment token config - no defaults, must be set
            payment_network: env::var("PAYMENT_NETWORK")
                .expect("PAYMENT_NETWORK must be set"),
            payment_token_address: env::var("PAYMENT_TOKEN_ADDRESS")
                .expect("PAYMENT_TOKEN_ADDRESS must be set"),
            payment_token_symbol: env::var("PAYMENT_TOKEN_SYMBOL")
                .expect("PAYMENT_TOKEN_SYMBOL must be set"),
            payment_token_decimals: env::var("PAYMENT_TOKEN_DECIMALS")
                .expect("PAYMENT_TOKEN_DECIMALS must be set")
                .parse()
                .expect("PAYMENT_TOKEN_DECIMALS must be a valid number"),
            payment_token_name: env::var("PAYMENT_TOKEN_NAME")
                .expect("PAYMENT_TOKEN_NAME must be set"),
            payment_token_version: env::var("PAYMENT_TOKEN_VERSION")
                .expect("PAYMENT_TOKEN_VERSION must be set"),
        }
    }
}
