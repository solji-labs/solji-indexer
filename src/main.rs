mod api_router;
mod db;
mod event_parser;
mod events;
mod idl;
mod indexer;
mod processor;
mod rewards;
mod utils;

use crate::api_router::*;
use crate::db::{create_pool, init_database};
use crate::events::ProgramEvent;
use crate::indexer::fetcher::IndexerFetcher;
use crate::processor::start_event_processor;
use crate::utils::config::Config;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // // 设置 panic hook 捕获所有 panic
    // std::panic::set_hook(Box::new(|panic_info| {
    //     eprintln!("PANIC: {:?}", panic_info);
    //     eprintln!("Location: {:?}", panic_info.location());
    // }));
    println!("Loaded config");
    // Load config
    let config = Config::from_env()?;
    println!(
        "Loaded config: RPC_URL={}, PROGRAM_ID={}, DATABASE_URL={}",
        config.rpc_url, config.program_id, config.database_url
    );

    // Initialize database
    println!("Initializing database...");
    let db_pool = create_pool(
        &config.database_url,
        config.db_max_open_conns,
        config.db_max_idle_conns,
        config.db_conn_max_lifetime,
    )
    .await?;
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

    let addr = "0.0.0.0:8080";
    println!("Starting HTTP server on {}", addr);
    println!(
        "📚 Swagger UI will be available at http://{}/swagger-ui",
        addr
    );
    println!("📖 OpenAPI spec at http://{}/api-docs/openapi.json", addr);

    // Add CORS middleware
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    // 将 create_router 作为起点，然后 merge Swagger 路由
    let app = create_router(state)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(cors);

    // Start the event listener in background
    tokio::spawn(async move {
        println!("Starting event listener...");
        if let Err(e) = fetcher.start_listening().await {
            eprintln!("Event listener error: {}", e);
        }
    });

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
