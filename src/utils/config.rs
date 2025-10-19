use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub program_id: Pubkey,
    pub database_url: String,
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_password: String,
    pub redis_db: i32,
    pub redis_pool_size: usize,
    pub redis_min_idle_conns: u8,
    pub redis_dial_timeout: u64,
    pub redis_read_timeout: u64,
    pub redis_write_timeout: u64,
    pub db_max_open_conns: u32,
    pub db_max_idle_conns: u32,
    pub db_conn_max_lifetime: u64,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv()?;

        let rpc_url = std::env::var("RPC_URL")?;
        let program_id_str = std::env::var("PROGRAM_ID")?;
        let program_id = Pubkey::from_str(&program_id_str)?;
        let database_url = std::env::var("DATABASE_URL")?;

        // Redis configuration
        let redis_host = std::env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string());
        let redis_port = std::env::var("REDIS_PORT")
            .unwrap_or_else(|_| "6379".to_string())
            .parse::<u16>()
            .map_err(|_| "Invalid REDIS_PORT")?;
        let redis_password = std::env::var("REDIS_PASSWORD").unwrap_or_default();
        let redis_db = std::env::var("REDIS_DB")
            .unwrap_or_else(|_| "0".to_string())
            .parse::<i32>()
            .map_err(|_| "Invalid REDIS_DB")?;
        let redis_pool_size = std::env::var("REDIS_POOL_SIZE")
            .unwrap_or_else(|_| "10".to_string())
            .parse::<usize>()
            .map_err(|_| "Invalid REDIS_POOL_SIZE")?;
        let redis_min_idle_conns = std::env::var("REDIS_MIN_IDLE_CONNS")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u8>()
            .map_err(|_| "Invalid REDIS_MIN_IDLE_CONNS")?;
        let redis_dial_timeout = std::env::var("REDIS_DIAL_TIMEOUT")
            .unwrap_or_else(|_| "5".to_string())
            .parse::<u64>()
            .map_err(|_| "Invalid REDIS_DIAL_TIMEOUT")?;
        let redis_read_timeout = std::env::var("REDIS_READ_TIMEOUT")
            .unwrap_or_else(|_| "3".to_string())
            .parse::<u64>()
            .map_err(|_| "Invalid REDIS_READ_TIMEOUT")?;
        let redis_write_timeout = std::env::var("REDIS_WRITE_TIMEOUT")
            .unwrap_or_else(|_| "3".to_string())
            .parse::<u64>()
            .map_err(|_| "Invalid REDIS_WRITE_TIMEOUT")?;

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
            redis_host,
            redis_port,
            redis_password,
            redis_db,
            redis_pool_size,
            redis_min_idle_conns,
            redis_dial_timeout,
            redis_read_timeout,
            redis_write_timeout,
            db_max_open_conns,
            db_max_idle_conns,
            db_conn_max_lifetime,
        })
    }
}
