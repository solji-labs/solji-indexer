use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::{
    check_daily_incense_limit, get_aggregated_global_stats,
    get_parsed_incense_leaderboard_by_period, get_wishes, update_incense_leaderboard_all_periods,
    DbPool,
};
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
        .route("/api/incense/leaderboard", get(get_incense_leaderboard))
        .route(
            "/api/admin/update-leaderboard",
            get(update_leaderboard_admin),
        )
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
    // Aggregate data from individual tables
    match get_aggregated_global_stats(&state.db_pool).await {
        Ok(stats) => {
            let response = json!({
                "total_merit": stats.total_merit,
                "total_incense_points": stats.total_incense_points,
                "total_donations_sol": stats.total_donations_sol,
                "total_users": stats.total_users,
                "total_wishes": stats.total_wishes,
                "total_donations": stats.total_donations,
                "total_donation_amount": stats.total_donation_amount,
                "total_merit_distributed": stats.total_merit_distributed,
                "total_incense_points_distributed": stats.total_incense_points_distributed,
                "total_draw_fortune": stats.total_draw_fortune,
                "updated_at": stats.updated_at.timestamp(),
                "created_at": stats.created_at.to_rfc3339()
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

async fn get_incense_leaderboard(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract query parameter with default
    let period = params.get("period").map(|s| s.as_str()).unwrap_or("all");

    // Validate period parameter
    let valid_periods = ["all", "daily", "weekly", "monthly"];
    if !valid_periods.contains(&period) {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Get leaderboard from database
    match get_parsed_incense_leaderboard_by_period(&state.db_pool, period).await {
        Ok(leaderboard) => {
            let leaderboard_data: Vec<serde_json::Value> = leaderboard
                .into_iter()
                .map(|entry| {
                    json!({
                        "rank": entry.rank,
                        "user_pubkey": entry.user_pubkey,
                        "total_incense_points": entry.total_incense_points,
                        "burn_count": entry.burn_count
                    })
                })
                .collect();

            let response = json!({
                "period": period,
                "leaderboard": leaderboard_data,
                "count": leaderboard_data.len()
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting incense leaderboard: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_leaderboard_admin(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Update all leaderboard periods
    match update_incense_leaderboard_all_periods(&state.db_pool, chrono::Utc::now()).await {
        Ok(()) => {
            let response = json!({
                "success": true,
                "message": "Incense leaderboard updated for all periods",
                "periods": ["all", "daily", "weekly", "monthly"]
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error updating leaderboard: {:?}", e);
            let response = json!({
                "success": false,
                "error": format!("Failed to update leaderboard: {:?}", e)
            });
            Ok(Json(response))
        }
    }
}
