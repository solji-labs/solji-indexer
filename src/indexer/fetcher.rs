use futures::{SinkExt, StreamExt};
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::Message;

use crate::event_parser::try_parse_event;
use crate::events::ProgramEvent;

pub struct IndexerFetcher {
    rpc_client: RpcClient,
    program_id: Pubkey,
    db_pool: Arc<Pool<MySql>>,
    event_sender: Sender<ProgramEvent>,
}

impl Clone for IndexerFetcher {
    fn clone(&self) -> Self {
        Self {
            rpc_client: RpcClient::new(self.rpc_client.url()),
            program_id: self.program_id,
            db_pool: self.db_pool.clone(),
            event_sender: self.event_sender.clone(),
        }
    }
}

impl IndexerFetcher {
    pub fn new(
        rpc_url: &str,
        program_id: Pubkey,
        db_pool: Arc<Pool<MySql>>,
        event_sender: Sender<ProgramEvent>,
    ) -> Self {
        let rpc_client = RpcClient::new(rpc_url.to_string());
        Self {
            rpc_client,
            program_id,
            db_pool,
            event_sender,
        }
    }

    // Core function: Start WebSocket listening and reconnection
    pub async fn start_listening(&self) -> Result<(), Box<dyn std::error::Error>> {
        let ws_url = self.rpc_url_to_ws_url();

        // Outer loop: Handle connection drops and reconnections
        loop {
            println!("Attempting to connect to WebSocket: {}", ws_url);
            match self.listen_for_logs(&ws_url).await {
                Ok(_) => {
                    println!("WebSocket listening finished normally (unexpected). Restarting...");
                }
                Err(e) => {
                    eprintln!("WebSocket connection failed or dropped: {:?}", e);
                }
            }

            // Exponential backoff retry (e.g., wait 5 seconds before reconnecting)
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    // Internal function: Lifecycle of a single connection
    async fn listen_for_logs(&self, ws_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Connect to WebSocket
        let (mut ws_stream, _) = tokio_tungstenite::connect_async(ws_url).await?;

        // 2. Subscribe to program logs
        let subscribe_message = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "logsSubscribe",
            "params": [
                { "mentions": [self.program_id.to_string()] },
                { "commitment": "finalized" }
            ]
        });

        ws_stream
            .send(Message::Text(subscribe_message.to_string()))
            .await?;
        println!("WebSocket subscribed to program logs.");

        // 3. Receive and process logs
        while let Some(msg_result) = ws_stream.next().await {
            let msg = msg_result?;
            if let Message::Text(text) = msg {
                println!("Received WebSocket message: {}", text);
                let json: Value = serde_json::from_str(&text)?;

                // Check for subscription confirmation first
                if json.get("result").is_some() && json.get("id").is_some() {
                    println!("WebSocket subscription confirmed");
                    continue;
                }

                // Parse notification messages with logs
                if let Some(log_data) = json
                    .get("params")
                    .and_then(|p| p.get("result"))
                    .and_then(|r| r.get("value"))
                    .and_then(|v| v.get("logs"))
                    .and_then(|l| l.as_array())
                {
                    println!("Found {} log entries in message", log_data.len());
                    for log_str_val in log_data {
                        if let Some(log_str) = log_str_val.as_str() {
                            println!("Processing log: {}", log_str);
                            // Core step: Parse and send to Channel
                            self.parse_and_send(log_str).await;
                        }
                    }
                } else {
                    println!("No logs array found in message");
                }
            } else {
                println!("Received non-text WebSocket message: {:?}", msg);
            }
        }

        Ok(()) // Normal disconnection (uncommon)
    }

    // 4. Parse and send function
    async fn parse_and_send(&self, log_str: &str) {
        // Use external module for parsing
        if let Some(parsed_event) = try_parse_event(log_str) {
            // Send parsed result to Worker thread
            if let Err(e) = self.event_sender.send(parsed_event).await {
                eprintln!("Failed to send event to channel: Channel closed or full. Worker failure? Error: {:?}", e);
                // Production environment may need more complex exit/alert logic
            }
        }
        // If parsing fails (returns None), silently ignore the log line
    }

    fn rpc_url_to_ws_url(&self) -> String {
        // Convert HTTP RPC URL to WebSocket URL
        // Special handling for local testnet: RPC on 8899, WebSocket on 8900
        if self.rpc_client.url().starts_with("http://127.0.0.1:8899")
            || self.rpc_client.url().starts_with("http://localhost:8899")
        {
            "ws://127.0.0.1:8900".to_string()
        } else if self.rpc_client.url().starts_with("https://") {
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
