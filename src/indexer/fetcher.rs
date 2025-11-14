use futures::{SinkExt, StreamExt};
use serde_json::Value;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::{interval, timeout, Instant};
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

    /// Starts WebSocket connection with automatic reconnection on failures.
    ///
    /// This is the main entry point for the indexer. It handles:
    /// - Initial connection establishment
    /// - Automatic reconnection with exponential backoff
    /// - Connection health monitoring
    ///
    /// The function runs indefinitely and will only return if explicitly stopped.
    pub async fn start_listening(&self) -> Result<(), String> {
        let ws_url = self.rpc_url_to_ws_url();
        let mut reconnect_delay = Duration::from_secs(2);
        let max_reconnect_delay = Duration::from_secs(60);
        let mut consecutive_failures = 0u32;

        loop {
            println!(
                "Attempting WebSocket connection: {} (delay: {:?}, failures: {})",
                ws_url, reconnect_delay, consecutive_failures
            );

            match self.listen_for_logs(&ws_url).await {
                Ok(_) => {
                    // Normal disconnection (rare) - reset backoff and retry immediately
                    println!("WebSocket connection closed normally, reconnecting...");
                    consecutive_failures = 0;
                    reconnect_delay = Duration::from_secs(2);
                }
                Err(e) => {
                    consecutive_failures += 1;
                    eprintln!(
                        "WebSocket error: {}. Failures: {}. Reconnecting in {:?}",
                        e, consecutive_failures, reconnect_delay
                    );

                    // Wait before reconnecting
                    tokio::time::sleep(reconnect_delay).await;

                    // Implement gradual backoff strategy:
                    // - First 5 failures: retry quickly (2s)
                    // - After 5 failures: gradually increase delay
                    if consecutive_failures < 5 {
                        reconnect_delay = Duration::from_secs(2);
                    } else {
                        reconnect_delay = std::cmp::min(
                            reconnect_delay + Duration::from_secs(5),
                            max_reconnect_delay,
                        );
                    }
                }
            }
        }
    }

    /// Processes incoming WebSocket messages and extracts log data.
    ///
    /// Handles different message types:
    /// - Subscription confirmations (initial handshake)
    /// - Log notifications containing transaction data
    ///
    /// Returns Ok(()) if message was processed successfully, even if no logs were found.
    async fn process_websocket_message(&self, text: &str) -> Result<(), String> {
        let json: Value =
            serde_json::from_str(text).map_err(|e| format!("Failed to parse JSON: {}", e))?;

        // Handle subscription confirmation response
        if json.get("result").is_some() && json.get("id").is_some() {
            println!("Subscription confirmed (id: {})", json["id"]);
            return Ok(());
        }

        // Extract transaction signature from the notification
        let transaction_signature = json
            .get("params")
            .and_then(|p| p.get("result"))
            .and_then(|r| r.get("value"))
            .and_then(|v| v.get("signature"))
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string();

        // Process log entries if present
        if let Some(log_data) = json
            .get("params")
            .and_then(|p| p.get("result"))
            .and_then(|r| r.get("value"))
            .and_then(|v| v.get("logs"))
            .and_then(|l| l.as_array())
        {
            println!(
                "Processing {} log entries from tx: {}",
                log_data.len(),
                &transaction_signature[..8.min(transaction_signature.len())]
            );

            for log_str_val in log_data {
                if let Some(log_str) = log_str_val.as_str() {
                    self.parse_and_send(log_str, &transaction_signature).await;
                }
            }
        }

        Ok(())
    }

    /// Manages a single WebSocket connection lifecycle.
    ///
    /// This function handles:
    /// - Initial connection with timeout
    /// - Subscription to program logs
    /// - Message processing loop with health checks
    /// - Graceful disconnection handling
    ///
    /// Returns Ok(()) on normal disconnection, Err() on connection failures.
    async fn listen_for_logs(&self, ws_url: &str) -> Result<(), String> {
        // Connect to WebSocket with timeout protection
        println!("Connecting to: {}", ws_url);
        let connect_future = tokio_tungstenite::connect_async(ws_url);
        let timeout_duration = Duration::from_secs(30);

        let (mut ws_stream, _) = match timeout(timeout_duration, connect_future).await {
            Ok(Ok(result)) => {
                println!("Connected successfully");
                result
            }
            Ok(Err(e)) => {
                return Err(format!("Connection failed: {}", e));
            }
            Err(_) => {
                return Err(format!("Connection timeout after {:?}", timeout_duration));
            }
        };

        // Subscribe to program logs with finalized commitment
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
            .await
            .map_err(|e| format!("Failed to send subscription: {}", e))?;

        println!("Subscribed to program logs");

        // Initialize connection health monitoring
        let mut ping_interval = interval(Duration::from_secs(30)); // Send ping every 30s
        let mut last_message_time = Instant::now();
        let message_timeout = Duration::from_secs(90); // Detect stale connection after 90s

        // Main message processing loop with concurrent health checks
        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                msg = ws_stream.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            last_message_time = Instant::now();

                            if let Err(e) = self.process_websocket_message(&text).await {
                                eprintln!("Error processing message: {:?}", e);
                                // Continue processing other messages despite errors
                            }
                        }
                        Some(Ok(Message::Ping(payload))) => {
                            last_message_time = Instant::now();
                            // Respond to server ping
                            if let Err(e) = ws_stream.send(Message::Pong(payload)).await {
                                eprintln!("Failed to send pong: {:?}", e);
                                break;
                            }
                        }
                        Some(Ok(Message::Pong(_))) => {
                            last_message_time = Instant::now();
                            // Pong received, connection is alive
                        }
                        Some(Ok(Message::Close(frame))) => {
                            println!("Server closed connection: {:?}", frame);
                            break;
                        }
                        Some(Ok(other)) => {
                            println!("Received unexpected message type: {:?}", other);
                        }
                        Some(Err(e)) => {
                            eprintln!("Stream error: {:?}", e);
                            break;
                        }
                        None => {
                            println!("Stream ended");
                            break;
                        }
                    }
                }

                // Send periodic ping to keep connection alive
                _ = ping_interval.tick() => {
                    if let Err(e) = ws_stream.send(Message::Ping(vec![])).await {
                        eprintln!("Failed to send ping: {:?}", e);
                        break;
                    }
                }

                // Detect stale connections (no messages received for too long)
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    if last_message_time.elapsed() > message_timeout {
                        eprintln!("No messages for {:?}, connection appears dead",
                            message_timeout);
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    // 4. Parse and send function
    async fn parse_and_send(&self, log_str: &str, transaction_signature: &str) {
        // Use external module for parsing
        if let Some(mut parsed_event) = try_parse_event(log_str) {
            // Add transaction signature to the event
            match &mut parsed_event {
                ProgramEvent::DonationCompleted {
                    transaction_signature: ts,
                    ..
                } => *ts = transaction_signature.to_string(),
                ProgramEvent::RewardsProcessed {
                    transaction_signature: ts,
                    ..
                } => *ts = transaction_signature.to_string(),
                ProgramEvent::DonationNFTMinted {
                    transaction_signature: ts,
                    ..
                } => *ts = transaction_signature.to_string(),
                ProgramEvent::FortuneDrawn {
                    transaction_signature: ts,
                    ..
                } => *ts = transaction_signature.to_string(),
                ProgramEvent::WishCreated {
                    transaction_signature: ts,
                    ..
                } => *ts = transaction_signature.to_string(),
                ProgramEvent::WishTowerUpdated {
                    transaction_signature: ts,
                    ..
                } => *ts = transaction_signature.to_string(),
                ProgramEvent::IncenseBurned {
                    transaction_signature: ts,
                    ..
                } => *ts = transaction_signature.to_string(),
                ProgramEvent::AmuletDropped {
                    transaction_signature: ts,
                    ..
                } => *ts = transaction_signature.to_string(),
                ProgramEvent::AmuletMinted {
                    transaction_signature: ts,
                    ..
                } => *ts = transaction_signature.to_string(),
                ProgramEvent::ShopConfigUpdated {
                    transaction_signature: ts,
                    ..
                } => *ts = transaction_signature.to_string(),
                ProgramEvent::FortuneNFTMinted {
                    transaction_signature: ts,
                    ..
                } => *ts = transaction_signature.to_string(),
            }

            println!(
                "✓ Parsed and sending event: {:?}",
                std::mem::discriminant(&parsed_event)
            );

            // Send parsed result to Worker thread
            if let Err(e) = self.event_sender.send(parsed_event).await {
                eprintln!(
                    "❌ CRITICAL: Failed to send event to channel: {:?}. Worker may have crashed!",
                    e
                );
            }
        }
    }

    /// Converts HTTP(S) RPC URL to WebSocket URL.
    ///
    /// Handles different URL schemes:
    /// - https:// -> wss://
    /// - http:// -> ws://
    /// - Special case: local testnet (8899 -> 8900)
    fn rpc_url_to_ws_url(&self) -> String {
        let url = self.rpc_client.url();

        // Special handling for local testnet: RPC on 8899, WebSocket on 8900
        if url.starts_with("http://127.0.0.1:8899") || url.starts_with("http://localhost:8899") {
            return "ws://127.0.0.1:8900".to_string();
        }

        // Standard HTTPS -> WSS conversion
        if url.starts_with("https://") {
            return url.replacen("https://", "wss://", 1);
        }

        // Standard HTTP -> WS conversion
        if url.starts_with("http://") {
            return url.replacen("http://", "ws://", 1);
        }

        // Fallback for URLs without scheme
        format!(
            "wss://{}",
            url.trim_start_matches("https://")
                .trim_start_matches("http://")
        )
    }
}
