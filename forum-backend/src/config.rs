use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub port: u16,
    pub facilitator_url: String,
    pub wallet_address: String,
    pub cost_per_registration: u64,
    pub cost_per_post: u64,
    // Payment token configuration
    pub payment_network: String,
    pub payment_token_address: String,
    pub payment_token_symbol: String,
    pub payment_token_decimals: u8,
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
            cost_per_registration: env::var("COST_PER_REGISTRATION")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .expect("COST_PER_REGISTRATION must be a valid number"),
            cost_per_post: env::var("COST_PER_POST")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .expect("COST_PER_POST must be a valid number"),
            // Payment token config - defaults to USDC on Base
            payment_network: env::var("PAYMENT_NETWORK")
                .unwrap_or_else(|_| "base".to_string()),
            payment_token_address: env::var("PAYMENT_TOKEN_ADDRESS")
                .unwrap_or_else(|_| "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913".to_string()),
            payment_token_symbol: env::var("PAYMENT_TOKEN_SYMBOL")
                .unwrap_or_else(|_| "USDC".to_string()),
            payment_token_decimals: env::var("PAYMENT_TOKEN_DECIMALS")
                .unwrap_or_else(|_| "6".to_string())
                .parse()
                .expect("PAYMENT_TOKEN_DECIMALS must be a valid number"),
        }
    }
}
