use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::{get_latest_global_stats, DbPool};
use crate::indexer::fetcher::IndexerFetcher;
use crate::utils::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub fetcher: Arc<RwLock<IndexerFetcher>>,
    pub config: Config,
    pub db_pool: DbPool,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/stats", get(get_global_stats))
        .route("/stats/global", get(get_global_stats))
        .with_state(state)
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "solji-indexer",
        "version": "0.1.0"
    }))
}

async fn get_global_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Query latest global stats from database
    match get_latest_global_stats(&state.db_pool).await {
        Ok(Some(stats)) => {
            let response = json!({
                "total_merit": stats.total_merit,
                "total_incense_points": stats.total_incense_points,
                "total_donations_sol": stats.total_donations_sol,
                "total_users": stats.total_users,
                "total_wishes": stats.total_wishes,
                "updated_at": stats.updated_at.timestamp(),
                "created_at": stats.created_at.to_rfc3339()
            });
            Ok(Json(response))
        }
        Ok(None) => {
            // No data found, return empty stats
            let response = json!({
                "total_merit": 0,
                "total_incense_points": 0,
                "total_donations_sol": 0.0,
                "total_users": 0,
                "total_wishes": 0,
                "updated_at": null,
                "created_at": null,
                "message": "No global stats found in database"
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
