use std::{str::FromStr, time::Duration};

use chrono::{DateTime, Local};
use dotenvy::dotenv;
use sol_ji_service::{
    business::{fetch_all_user_info, fetch_burn_incense_type_once, fetch_temple_once},
    infra::connect_db,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use tokio::time::interval;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let local_now: DateTime<Local> = Local::now();
    println!("start time: {}", local_now);
    dotenv().ok();
    let rpc_url = std::env::var("RPC_URL")?;
    let program_id = std::env::var("PROGRAM_ID")?;
    let id = Pubkey::from_str(&program_id)?;
    let rpc = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::finalized());
    let pool = connect_db().await?;
    fetch_all_user_info(&rpc, id, &pool).await?;
    fetch_burn_incense_type_once(&rpc, id, &pool).await?;
    fetch_temple_once(&rpc, id, &pool).await?;
    let mut ticker = interval(Duration::from_secs(5 * 60));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if let Err(e) = fetch_all_user_info(&rpc, id,&pool).await {
                    eprintln!("[fetch_all_user_info] {e:?}");
                }
                if let Err(e) = fetch_temple_once(&rpc, id,&pool).await {
                    eprintln!("[fetch_temple_once] {e:?}");
                }
                if let Err(e) = fetch_burn_incense_type_once(&rpc, id,&pool).await {
                    eprintln!("[fetch_burn_incense_type_once] {e:?}");
                }
            }
            _ = tokio::signal::ctrl_c() => {
                eprintln!("ctrl-c, bye");
                break;
            }
        }
    }
    Ok(())
}
