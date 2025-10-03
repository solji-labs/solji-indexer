mod api;
mod db;
mod event_parser;
mod events;
mod indexer;
mod test_db;
mod utils;

use crate::api::{create_router, AppState};
use crate::db::{create_pool, init_database};
use crate::events::ProgramEvent;
use crate::indexer::fetcher::IndexerFetcher;
use crate::utils::config::Config;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config
    let config = Config::from_env()?;
    println!(
        "Loaded config: RPC_URL={}, PROGRAM_ID={}, DATABASE_URL={}",
        config.rpc_url, config.program_id, config.database_url
    );

    // Initialize database
    println!("Initializing database...");
    let db_pool = create_pool(&config.database_url).await?;
    init_database(&db_pool).await?;
    println!("Database initialized successfully");

    // Create event channel
    let (event_sender, _event_receiver) = mpsc::channel::<ProgramEvent>(100);

    // Create fetcher
    let fetcher = IndexerFetcher::new(
        &config.rpc_url,
        config.program_id,
        db_pool.clone(),
        event_sender,
    );
    let fetcher = Arc::new(RwLock::new(fetcher));

    // Create app state
    let state = AppState {
        fetcher: fetcher.clone(),
        config: config.clone(),
        db_pool: db_pool.clone(),
    };

    let app = create_router(state);

    let addr = "0.0.0.0:3000";
    println!("Starting HTTP server on {}", addr);

    // Start the event listener
    let fetcher_clone = fetcher.clone();
    tokio::spawn(async move {
        let fetcher = fetcher_clone.read().await;
        println!("Starting event listener...");
        if let Err(e) = fetcher.start_listening().await {
            eprintln!("Event listener error: {:?}", e);
        }
    });

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
