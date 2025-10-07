use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::{check_daily_incense_limit, get_latest_global_stats, get_wishes, DbPool};
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
        .route("/api/incense/can-burn", get(check_can_burn_incense))
        .route("/api/wishes", get(get_wishes_paginated))
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

async fn check_can_burn_incense(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract query parameters
    let user_pubkey = params.get("user").ok_or(StatusCode::BAD_REQUEST)?;
    let incense_type_str = params.get("incense_type").ok_or(StatusCode::BAD_REQUEST)?;
    let amount_str = params.get("amount").ok_or(StatusCode::BAD_REQUEST)?;

    let incense_type: i32 = incense_type_str
        .parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let amount: i32 = amount_str.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    // Check daily limit
    match check_daily_incense_limit(&state.db_pool, user_pubkey, incense_type, amount).await {
        Ok(can_burn) => {
            let response = json!({
                "can_burn": can_burn,
                "user": user_pubkey,
                "incense_type": incense_type,
                "requested_amount": amount,
                "max_daily_limit": 10
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error checking incense limit: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_wishes_paginated(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract query parameters with defaults
    let limit_str = params.get("limit").map(|s| s.as_str()).unwrap_or("20");
    let offset_str = params.get("offset").map(|s| s.as_str()).unwrap_or("0");

    let limit: i32 = limit_str.parse().unwrap_or(20).min(100); // Max 100 per page
    let offset: i32 = offset_str.parse().unwrap_or(0).max(0);

    // Get wishes from database
    match get_wishes(&state.db_pool, limit, offset).await {
        Ok(wishes) => {
            let wishes_data: Vec<serde_json::Value> = wishes
                .into_iter()
                .map(|wish| {
                    json!({
                        "id": wish.id,
                        "wish_id": wish.wish_id,
                        "user_pubkey": wish.user_pubkey,
                        "content": wish.content,
                        "likes": wish.likes,
                        "created_at": wish.created_at.to_rfc3339(),
                        "updated_at": wish.updated_at.to_rfc3339()
                    })
                })
                .collect();

            let response = json!({
                "wishes": wishes_data,
                "pagination": {
                    "limit": limit,
                    "offset": offset,
                    "count": wishes_data.len()
                }
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting wishes: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
