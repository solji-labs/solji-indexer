use serde::Deserialize;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rpc_url: String,
    pub program_id: Pubkey,
    pub database_url: String,
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Config {
            rpc_url: self.rpc_url.clone(),
            program_id: self.program_id,
            database_url: self.database_url.clone(),
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv()?;

        let rpc_url = std::env::var("RPC_URL")?;
        let program_id_str = std::env::var("PROGRAM_ID")?;
        let program_id = Pubkey::from_str(&program_id_str)?;
        let database_url = std::env::var("DATABASE_URL")?;

        Ok(Config {
            rpc_url,
            program_id,
            database_url,
        })
    }
}
