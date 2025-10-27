// api/fortune.rs

use axum::{extract::Path, extract::Query, http::StatusCode, response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::AppState;
use crate::db::{
    get_fortune_leaderboard, get_user_fortune_draw_history, get_user_fortune_nft_mints,
    get_user_fortune_stats,
};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/fortune/leaderboard",
            get(get_fortune_leaderboard_handler),
        )
        .route(
            "/api/fortune/user/{user_pubkey}/history",
            get(get_user_fortune_history_handler),
        )
        .route(
            "/api/fortune/user/{user_pubkey}/mints",
            get(get_user_fortune_nft_mints_handler),
        )
        .route(
            "/api/fortune/user/{user_pubkey}/stats",
            get(get_user_fortune_stats_handler),
        )
        .with_state(state)
}

/// Query parameters for leaderboard
#[derive(Debug, Deserialize)]
pub struct LeaderboardQuery {
    pub limit: Option<i32>,
}

/// Get fortune leaderboard
#[utoipa::path(
    get,
    path = "/api/fortune/leaderboard",
    params(
        ("limit" = Option<i32>, Query, description = "Maximum number of entries to return (default: 100, max: 1000)")
    ),
    responses(
        (status = 200, description = "Fortune leaderboard", body = serde_json::Value)
    ),
    tag = "Fortune"
)]
pub async fn get_fortune_leaderboard_handler(
    Query(params): Query<LeaderboardQuery>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = params.limit.unwrap_or(100).min(1000).max(1);

    match get_fortune_leaderboard(&state.db_pool, limit).await {
        Ok(leaderboard) => {
            let count = leaderboard.len() as i32;
            let data: Vec<_> = leaderboard
                .into_iter()
                .enumerate()
                .map(|(i, entry)| {
                    json!({
                        "rank": i + 1,
                        "user_pubkey": entry.user_pubkey,
                        "total_draws": entry.total_draws
                    })
                })
                .collect();

            Ok(Json(json!({
                "leaderboard": data,
                "count": count
            })))
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user's fortune draw history
#[utoipa::path(
    get,
    path = "/api/fortune/user/{user_pubkey}/history",
    params(
        ("user_pubkey" = String, Path, description = "User's public key"),
        ("limit" = Option<i32>, Query, description = "Maximum number of entries to return (default: 50, max: 100)")
    ),
    responses(
        (status = 200, description = "User's fortune draw history", body = serde_json::Value)
    ),
    tag = "Fortune"
)]
pub async fn get_user_fortune_history_handler(
    Path(user_pubkey): Path<String>,
    Query(params): Query<HistoryQuery>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = params.limit.unwrap_or(50).min(100).max(1);

    match get_user_fortune_draw_history(&state.db_pool, &user_pubkey, limit).await {
        Ok(history) => {
            let count = history.len() as i32;
            let data: Vec<_> = history
                .into_iter()
                .map(|r| {
                    json!({
                        "id": r.id,
                        "fortune_text": r.fortune_text,
                        "merit_cost": r.merit_cost,
                        "is_free": r.is_free,
                        "transaction_signature": r.transaction_signature,
                        "created_at": r.created_at.to_rfc3339()
                    })
                })
                .collect();

            Ok(Json(json!({
                "user_pubkey": user_pubkey,
                "history": data,
                "count": count
            })))
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user's fortune NFT mints
#[utoipa::path(
    get,
    path = "/api/fortune/user/{user_pubkey}/mints",
    params(
        ("user_pubkey" = String, Path, description = "User's public key"),
        ("limit" = Option<i32>, Query, description = "Maximum number of entries to return (default: 50, max: 100)")
    ),
    responses(
        (status = 200, description = "User's fortune NFT mints", body = serde_json::Value)
    ),
    tag = "Fortune"
)]
pub async fn get_user_fortune_nft_mints_handler(
    Path(user_pubkey): Path<String>,
    Query(params): Query<HistoryQuery>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = params.limit.unwrap_or(50).min(100).max(1);

    match get_user_fortune_nft_mints(&state.db_pool, &user_pubkey, limit).await {
        Ok(mints) => {
            let count = mints.len() as i32;
            let data: Vec<_> = mints
                .into_iter()
                .map(|r| {
                    json!({
                        "id": r.id,
                        "fortune_nft_mint": r.fortune_nft_mint,
                        "fortune_result": r.fortune_result,
                        "merit_cost": r.merit_cost,
                        "serial_number": r.serial_number,
                        "created_at": r.created_at.to_rfc3339()
                    })
                })
                .collect();

            Ok(Json(json!({
                "user_pubkey": user_pubkey,
                "mints": data,
                "count": count
            })))
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user's fortune statistics
#[utoipa::path(
    get,
    path = "/api/fortune/user/{user_pubkey}/stats",
    params(
        ("user_pubkey" = String, Path, description = "User's public key")
    ),
    responses(
        (status = 200, description = "User's fortune statistics", body = serde_json::Value)
    ),
    tag = "Fortune"
)]
pub async fn get_user_fortune_stats_handler(
    Path(user_pubkey): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match get_user_fortune_stats(&state.db_pool, &user_pubkey).await {
        Ok(stats) => {
            let recent_draws: Vec<_> = stats
                .recent_draws
                .into_iter()
                .map(|r| {
                    json!({
                        "id": r.id,
                        "fortune_text": r.fortune_text,
                        "merit_cost": r.merit_cost,
                        "is_free": r.is_free,
                        "transaction_signature": r.transaction_signature,
                        "created_at": r.created_at.to_rfc3339()
                    })
                })
                .collect();

            Ok(Json(json!({
                "user_pubkey": stats.user_pubkey,
                "total_draws": stats.total_draws,
                "fortune_nft_count": stats.fortune_nft_count,
                "rank": stats.rank,
                "recent_draws": recent_draws
            })))
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ===== QUERY PARAMETERS =====

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i32>,
}

// ===== RESPONSE STRUCTS =====

#[derive(Debug, Serialize)]
pub struct FortuneLeaderboardResponse {
    pub leaderboard: Vec<FortuneLeaderboardEntryWithRank>,
    pub count: i32,
}

#[derive(Debug, Serialize)]
pub struct FortuneLeaderboardEntryWithRank {
    pub rank: i32,
    pub user_pubkey: String,
    pub total_draws: i32,
}

#[derive(Debug, Serialize)]
pub struct UserFortuneHistoryResponse {
    pub user_pubkey: String,
    pub history: Vec<crate::db::models::FortuneDrawHistory>,
    pub count: i32,
}

#[derive(Debug, Serialize)]
pub struct UserFortuneNFTMintsResponse {
    pub user_pubkey: String,
    pub mints: Vec<crate::db::models::FortuneNftMintHistory>,
    pub count: i32,
}
