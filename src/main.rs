mod api;
mod db;
mod event_parser;
mod events;
mod indexer;
mod processor;
mod rewards;
mod test_db;
mod utils;

use crate::api::{create_router, AppState};
use crate::db::{create_pool, init_database, update_incense_leaderboard_all_periods};
use crate::events::ProgramEvent;
use crate::indexer::fetcher::IndexerFetcher;
use crate::processor::start_event_processor;
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

    // Create event channel with larger buffer
    let (event_sender, mut event_receiver) = mpsc::channel::<ProgramEvent>(1024);

    // Start multiple worker threads for event processing
    let num_workers = 4; // Adjust based on CPU cores and database performance
    let mut worker_senders = Vec::new();

    for i in 0..num_workers {
        let (worker_sender, worker_receiver) = mpsc::channel::<ProgramEvent>(100);
        let pool_clone = db_pool.clone();

        worker_senders.push(worker_sender);

        tokio::spawn(async move {
            // Worker loop: continuously receive events from channel
            start_event_processor(i, worker_receiver, pool_clone).await;
        });
    }

    // Start a distributor task that reads from the main channel and distributes to workers
    let distributor_pool = db_pool.clone();
    tokio::spawn(async move {
        let mut worker_index = 0;
        while let Some(event) = event_receiver.recv().await {
            // Round-robin distribution to workers
            if let Err(_) = worker_senders[worker_index].send(event).await {
                eprintln!("Failed to send event to worker {}", worker_index);
            }
            worker_index = (worker_index + 1) % num_workers;
        }
    });

    // Create fetcher
    let fetcher = IndexerFetcher::new(
        &config.rpc_url,
        config.program_id,
        db_pool.clone(),
        event_sender,
    );

    // Create app state
    let state = AppState {
        fetcher: Arc::new(RwLock::new(fetcher.clone())),
        config: config.clone(),
        db_pool: db_pool.clone(),
    };

    let app = create_router(state);

    let addr = "0.0.0.0:3000";
    println!("Starting HTTP server on {}", addr);

    // Start the event listener in background
    tokio::spawn(async move {
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
