use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub program_id: Pubkey,
    pub database_url: String,
    pub db_max_open_conns: u32,
    pub db_max_idle_conns: u32,
    pub db_conn_max_lifetime: u64,
    pub pinata_jwt: String,
    pub pinata_api_url: String,
    pub pinata_gateway: String,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        match dotenvy::dotenv() {
            Ok(_) => println!("Successfully loaded .env file from disk."),
            Err(e) => println!(
                "Could not load .env file from disk (expected in Docker): {}",
                e
            ),
        };

        let rpc_url = std::env::var("RPC_URL")?;
        let program_id_str = std::env::var("PROGRAM_ID")?;
        let program_id = Pubkey::from_str(&program_id_str)?;
        let database_url = std::env::var("DATABASE_URL")?;
        let pinata_jwt = std::env::var("PINATA_JWT")?;
        let pinata_api_url = std::env::var("PINATA_API_URL")?;
        let pinata_gateway = std::env::var("PINATA_GATEWAY")?;

        // Database connection pool settings
        let db_max_open_conns = std::env::var("DB_MAX_OPEN_CONNS")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<u32>()
            .map_err(|_| "Invalid DB_MAX_OPEN_CONNS")?;
        let db_max_idle_conns = std::env::var("DB_MAX_IDLE_CONNS")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<u32>()
            .map_err(|_| "Invalid DB_MAX_IDLE_CONNS")?;
        let db_conn_max_lifetime = std::env::var("DB_CONN_MAX_LIFETIME")
            .unwrap_or_else(|_| "3600".to_string())
            .parse::<u64>()
            .map_err(|_| "Invalid DB_CONN_MAX_LIFETIME")?;

        Ok(Config {
            rpc_url,
            program_id,
            database_url,
            db_max_open_conns,
            db_max_idle_conns,
            db_conn_max_lifetime,
            pinata_jwt,
            pinata_api_url,
            pinata_gateway,
        })
    }
}
