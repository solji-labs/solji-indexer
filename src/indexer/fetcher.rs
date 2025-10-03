use anchor_lang::AccountDeserialize;
use futures_util::{SinkExt, StreamExt};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use std::time::Duration;
use temple::state::global_stats::GlobalStats;
use tokio::time;

pub struct IndexerFetcher {
    rpc_client: RpcClient,
    program_id: Pubkey,
    db_pool: Arc<Pool<MySql>>,
}

impl IndexerFetcher {
    pub fn new(rpc_url: &str, program_id: Pubkey, db_pool: Arc<Pool<MySql>>) -> Self {
        let rpc_client = RpcClient::new(rpc_url.to_string());
        Self {
            rpc_client,
            program_id,
            db_pool,
        }
    }

    pub async fn start_polling(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut interval = time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;
            self.fetch_global_stats().await?;
        }
    }

    async fn fetch_global_stats(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (global_stats_pda, _) =
            Pubkey::find_program_address(&[b"global_stats_v1"], &self.program_id);

        match self.rpc_client.get_account(&global_stats_pda) {
            Ok(account) => match GlobalStats::try_deserialize(&mut &account.data[..]) {
                Ok(global_stats) => {
                    println!("Fetched GlobalStats successfully:");
                    println!("  Total Merit: {}", global_stats.total_merit);
                    println!(
                        "  Total Incense Points: {}",
                        global_stats.total_incense_points
                    );
                    println!(
                        "  Total Donations (SOL): {:.9}",
                        global_stats.total_donations_sol()
                    );
                    println!("  Total Users: {}", global_stats.total_users);
                    println!("  Total Wishes: {}", global_stats.total_wishes);
                    println!("  Updated At: {}", global_stats.updated_at);
                }
                Err(e) => {
                    eprintln!("Failed to deserialize GlobalStats: {:?}", e);
                }
            },
            Err(e) => {
                eprintln!("Failed to fetch GlobalStats account: {:?}", e);
            }
        }

        Ok(())
    }

    pub async fn start_event_listener(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Starting event listener for program: {}", self.program_id);

        // Convert HTTP URL to WebSocket URL
        let ws_url = self.rpc_url_to_ws_url();

        match tokio_tungstenite::connect_async(&ws_url).await {
            Ok((ws_stream, _)) => {
                println!("WebSocket connection established to: {}", ws_url);

                let (mut write, mut read) = ws_stream.split();

                // Send subscription request
                let subscribe_msg = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "logsSubscribe",
                    "params": [
                        {
                            "mentions": [self.program_id.to_string()]
                        },
                        {
                            "commitment": "confirmed"
                        }
                    ]
                });

                let msg = tokio_tungstenite::tungstenite::Message::Text(subscribe_msg.to_string());
                if let Err(e) = write.send(msg).await {
                    eprintln!("Failed to send subscription message: {:?}", e);
                    return Ok(());
                }

                println!("Subscribed to program logs for: {}", self.program_id);

                // Listen for messages
                while let Some(message) = read.next().await {
                    match message {
                        Ok(msg) => {
                            if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text)
                                {
                                    if let Some(logs) = parsed
                                        .get("params")
                                        .and_then(|p| p.get("result"))
                                        .and_then(|r| r.get("value"))
                                        .and_then(|v| v.get("logs"))
                                        .and_then(|l| l.as_array())
                                    {
                                        for log in logs {
                                            if let Some(log_str) = log.as_str() {
                                                println!("Received program log: {}", log_str);
                                                // TODO: Parse and handle Anchor events
                                                // For now, just log the raw event
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("WebSocket error: {:?}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to connect to WebSocket: {:?}", e);
                println!("Falling back to polling mode...");
            }
        }

        Ok(())
    }

    fn rpc_url_to_ws_url(&self) -> String {
        // Convert HTTP RPC URL to WebSocket URL
        if self.rpc_client.url().starts_with("https://") {
            self.rpc_client.url().replacen("https://", "wss://", 1)
        } else if self.rpc_client.url().starts_with("http://") {
            self.rpc_client.url().replacen("http://", "ws://", 1)
        } else {
            // Default fallback
            format!(
                "wss://{}",
                self.rpc_client
                    .url()
                    .trim_start_matches("https://")
                    .trim_start_matches("http://")
            )
        }
    }
}
