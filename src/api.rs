use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::{
    check_daily_incense_limit, check_user_in_top_10000_donors, get_aggregated_global_stats,
    get_donation_leaderboard as get_donation_leaderboard_db,
    get_parsed_incense_leaderboard_by_period, get_public_wishes, get_user_daily_wish_count,
    get_user_donation_badges, get_user_donation_history, get_user_incense_burn_count,
    get_user_incense_burn_history, get_user_incense_nfts, get_user_wish_tower_stats,
    get_user_wishes, get_wishes, like_wish_by_id, process_donation_transaction,
    update_incense_leaderboard_all_periods, DbPool,
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
        .route("/api/incense/can-burn", get(check_can_burn_incense))
        .route("/api/incense/types", get(get_incense_types))
        .route(
            "/api/incense/user/{user_pubkey}/burn-count",
            get(get_user_incense_burn_count_api),
        )
        .route(
            "/api/incense/user/{user_pubkey}/nfts",
            get(get_user_incense_nfts_api),
        )
        .route(
            "/api/incense/user/{user_pubkey}/history",
            get(get_user_incense_burn_history_api),
        )
        .route("/api/wishes", get(get_wishes_paginated))
        .route("/api/wishes/public", get(get_public_wishes_api))
        .route("/api/wishes/user/{user_pubkey}", get(get_user_wishes_api))
        .route(
            "/api/wishes/user/{user_pubkey}/count",
            get(get_user_daily_wish_count_api),
        )
        .route(
            "/api/wish-tower/{user_pubkey}",
            get(get_user_wish_tower_api),
        )
        .route(
            "/api/wishes/{wish_id}/like",
            axum::routing::post(like_wish_api),
        )
        .route("/api/incense/leaderboard", get(get_incense_leaderboard))
        .route(
            "/api/donation/leaderboard",
            get(get_donation_leaderboard_api),
        )
        .route(
            "/api/donation/check-top-10000",
            get(check_user_top_10000_donors),
        )
        .route("/api/donation/tiers", get(get_donation_tiers_api))
        .route(
            "/api/donation/user/{user_pubkey}/history",
            get(get_user_donation_history_api),
        )
        .route(
            "/api/donation/user/{user_pubkey}/badges",
            get(get_user_donation_badges_api),
        )
        .route("/api/donation/honor-wall", get(get_honor_wall_api))
        .route(
            "/api/donation/submit",
            axum::routing::post(submit_donation_transaction_api),
        )
        .route(
            "/api/admin/update-leaderboard",
            get(update_leaderboard_admin),
        )
        .route(
            "/api/amulet/user/{user_pubkey}/pending",
            get(get_user_pending_amulets),
        )
        .route(
            "/api/amulet/user/{user_pubkey}/recent-drop",
            get(get_user_recent_amulet_drop),
        )
        .route(
            "/api/amulet/user/{user_pubkey}/owned",
            get(get_user_owned_amulets),
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

async fn get_donation_leaderboard_api(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract query parameters with defaults
    let limit_str = params.get("limit").map(|s| s.as_str()).unwrap_or("100");
    let offset_str = params.get("offset").map(|s| s.as_str()).unwrap_or("0");

    let limit: usize = limit_str.parse().unwrap_or(100).min(1000); // Max 1000 per page
    let offset: usize = offset_str.parse().unwrap_or(0).max(0);

    // Get leaderboard from database (already paginated)
    match get_donation_leaderboard_db(&state.db_pool, limit, offset).await {
        Ok(leaderboard) => {
            let leaderboard_data: Vec<serde_json::Value> = leaderboard
                .into_iter()
                .map(|entry| {
                    json!({
                        "rank": entry.rank,
                        "user_pubkey": entry.user_pubkey,
                        "total_donated": entry.total_donated
                    })
                })
                .collect();

            let response = json!({
                "leaderboard": leaderboard_data,
                "pagination": {
                    "limit": limit,
                    "offset": offset,
                    "count": leaderboard_data.len()
                }
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting donation leaderboard: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn check_user_top_10000_donors(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract query parameter
    let user_pubkey = params.get("user").ok_or(StatusCode::BAD_REQUEST)?;

    // Check if user is in top 10000 donors
    match check_user_in_top_10000_donors(&state.db_pool, user_pubkey).await {
        Ok(is_in_top_10000) => {
            let response = json!({
                "user_pubkey": user_pubkey,
                "is_in_top_10000": is_in_top_10000
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error checking user top 10000 donors: {:?}", e);
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

// ===== WISH-RELATED API ENDPOINTS =====

async fn get_public_wishes_api(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract query parameters with defaults
    let limit_str = params.get("limit").map(|s| s.as_str()).unwrap_or("20");
    let offset_str = params.get("offset").map(|s| s.as_str()).unwrap_or("0");

    let limit: i32 = limit_str.parse().unwrap_or(20).min(100); // Max 100 per page
    let offset: i32 = offset_str.parse().unwrap_or(0).max(0);

    // Get public wishes from database
    match get_public_wishes(&state.db_pool, limit, offset).await {
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
            eprintln!("Database error getting public wishes: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_user_wishes_api(
    State(state): State<AppState>,
    axum::extract::Path(user_pubkey): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract query parameters with defaults
    let limit_str = params.get("limit").map(|s| s.as_str()).unwrap_or("20");
    let offset_str = params.get("offset").map(|s| s.as_str()).unwrap_or("0");

    let limit: i32 = limit_str.parse().unwrap_or(20).min(100); // Max 100 per page
    let offset: i32 = offset_str.parse().unwrap_or(0).max(0);

    // Get user's wishes from database
    match get_user_wishes(&state.db_pool, &user_pubkey, limit, offset).await {
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
                "user_pubkey": user_pubkey,
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
            eprintln!("Database error getting user wishes: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_user_daily_wish_count_api(
    State(state): State<AppState>,
    axum::extract::Path(user_pubkey): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get user's daily wish count
    match get_user_daily_wish_count(&state.db_pool, &user_pubkey).await {
        Ok(count) => {
            let response = json!({
                "user_pubkey": user_pubkey,
                "daily_wish_count": count,
                "max_daily_limit": 3
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting user daily wish count: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_user_wish_tower_api(
    State(state): State<AppState>,
    axum::extract::Path(user_pubkey): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get user's wish tower stats
    match get_user_wish_tower_stats(&state.db_pool, &user_pubkey).await {
        Ok(tower_stats) => {
            let response = json!({
                "user_pubkey": tower_stats.user_pubkey,
                "total_wishes": tower_stats.total_wishes,
                "level": tower_stats.level,
                "last_updated": tower_stats.last_updated.map(|dt| dt.to_rfc3339())
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting user wish tower: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn like_wish_api(
    State(state): State<AppState>,
    axum::extract::Path(wish_id): axum::extract::Path<i64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Like the wish and get updated count
    match like_wish_by_id(&state.db_pool, wish_id).await {
        Ok(new_likes_count) => {
            let response = json!({
                "wish_id": wish_id,
                "likes": new_likes_count,
                "success": true
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error liking wish: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ===== INCENSE-RELATED API ENDPOINTS =====

#[derive(Debug, Serialize, Deserialize)]
pub struct IncenseTypeInfo {
    pub id: String,
    pub name: String,
    pub name_en: String,
    pub price: f64,
    pub merit_points: i32,
    pub description: String,
    pub image: String,
    pub daily_limit: i32,
}

async fn get_incense_types() -> Result<Json<serde_json::Value>, StatusCode> {
    // Return hardcoded incense types based on frontend constants
    let incense_types = vec![
        IncenseTypeInfo {
            id: "basic".to_string(),
            name: "清香".to_string(),
            name_en: "Basic Incense".to_string(),
            price: 0.01,
            merit_points: 1,
            description: "Simple and pure, for daily devotion".to_string(),
            image: "/traditional-incense-stick-glowing.jpg".to_string(),
            daily_limit: 10,
        },
        IncenseTypeInfo {
            id: "sandalwood".to_string(),
            name: "檀香".to_string(),
            name_en: "Sandalwood".to_string(),
            price: 0.05,
            merit_points: 5,
            description: "Premium sandalwood for deeper meditation".to_string(),
            image: "/sandalwood-incense-with-golden-glow.jpg".to_string(),
            daily_limit: 10,
        },
        IncenseTypeInfo {
            id: "dragon".to_string(),
            name: "龙香".to_string(),
            name_en: "Dragon Incense".to_string(),
            price: 0.1,
            merit_points: 10,
            description: "Rare dragon incense for great fortune".to_string(),
            image: "/mystical-dragon-incense-with-purple-smoke.jpg".to_string(),
            daily_limit: 10,
        },
        IncenseTypeInfo {
            id: "supreme".to_string(),
            name: "至尊香".to_string(),
            name_en: "Supreme Incense".to_string(),
            price: 0.3,
            merit_points: 30,
            description: "The ultimate offering for enlightenment".to_string(),
            image: "/supreme-golden-incense-with-rainbow-aura.jpg".to_string(),
            daily_limit: 10,
        },
    ];

    let response = json!({
        "incense_types": incense_types
    });
    Ok(Json(response))
}

async fn get_user_incense_burn_count_api(
    State(state): State<AppState>,
    axum::extract::Path(user_pubkey): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get user's daily burn count for all incense types
    match get_user_incense_burn_count(&state.db_pool, &user_pubkey).await {
        Ok(burn_counts) => {
            let response = json!({
                "user_pubkey": user_pubkey,
                "burn_counts": burn_counts,
                "max_daily_limit": 10
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting user incense burn count: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_user_incense_nfts_api(
    State(state): State<AppState>,
    axum::extract::Path(user_pubkey): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get user's incense NFTs
    match get_user_incense_nfts(&state.db_pool, &user_pubkey).await {
        Ok(nfts) => {
            let response = json!({
                "user_pubkey": user_pubkey,
                "nfts": nfts
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting user incense NFTs: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_user_incense_burn_history_api(
    State(state): State<AppState>,
    axum::extract::Path(user_pubkey): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract query parameters with defaults
    let limit_str = params.get("limit").map(|s| s.as_str()).unwrap_or("20");

    let limit: i32 = limit_str.parse().unwrap_or(20).min(100); // Max 100 per page

    // Get user's incense burn history
    match get_user_incense_burn_history(&state.db_pool, &user_pubkey, limit).await {
        Ok(history) => {
            let history_data: Vec<serde_json::Value> = history
                .into_iter()
                .map(|record| {
                    json!({
                        "id": record.id,
                        "user_pubkey": record.user_pubkey,
                        "incense_type": record.incense_type,
                        "incense_amount": record.incense_amount,
                        "merit_gained": record.merit_gained,
                        "incense_points_gained": record.incense_points_gained,
                        "transaction_signature": record.transaction_signature,
                        "created_at": record.created_at.to_rfc3339()
                    })
                })
                .collect();

            let response = json!({
                "user_pubkey": user_pubkey,
                "history": history_data,
                "count": history_data.len()
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting user incense burn history: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ===== DONATION API ENDPOINTS =====

#[derive(Debug, Serialize, Deserialize)]
pub struct DonationTierInfo {
    pub tier: String,
    pub name: String,
    pub name_en: String,
    pub min_amount: f64,
    pub merit_points: i32,
    pub badge: String,
    pub benefits: Vec<String>,
}

async fn get_donation_tiers_api() -> Result<Json<serde_json::Value>, StatusCode> {
    // Return donation tiers based on the official specification
    let tiers = vec![
        DonationTierInfo {
            tier: "bronze".to_string(),
            name: "铜牌信士".to_string(),
            name_en: "Bronze Devotee".to_string(),
            min_amount: 0.05,
            merit_points: 65,
            badge: "入门功德铜章 NFT".to_string(),
            benefits: vec!["点亮香火墙名字".to_string()],
        },
        DonationTierInfo {
            tier: "silver".to_string(),
            name: "银牌居士".to_string(),
            name_en: "Silver Layman".to_string(),
            min_amount: 0.2,
            merit_points: 1300,
            badge: "精进银章 NFT".to_string(),
            benefits: vec!["可为寺庙投票提案".to_string()],
        },
        DonationTierInfo {
            tier: "gold".to_string(),
            name: "金牌护法".to_string(),
            name_en: "Gold Guardian".to_string(),
            min_amount: 1.0,
            merit_points: 14000,
            badge: "护法金章 NFT".to_string(),
            benefits: vec!["可参与寺庙 NFT 治理".to_string()],
        },
        DonationTierInfo {
            tier: "supreme".to_string(),
            name: "至尊供奉".to_string(),
            name_en: "Supreme Patron".to_string(),
            min_amount: 5.0,
            merit_points: 120000,
            badge: "至尊龙章 NFT".to_string(),
            benefits: vec!["解锁彩蛋内容+寺庙共建者身份".to_string()],
        },
    ];

    let response = json!({
        "tiers": tiers
    });
    Ok(Json(response))
}

async fn get_user_donation_history_api(
    State(state): State<AppState>,
    axum::extract::Path(user_pubkey): axum::extract::Path<String>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract query parameters with defaults
    let limit_str = params.get("limit").map(|s| s.as_str()).unwrap_or("20");

    let limit: i32 = limit_str.parse().unwrap_or(20).min(100); // Max 100 per page

    // Get user's donation history from database
    match get_user_donation_history(&state.db_pool, &user_pubkey, limit).await {
        Ok(history) => {
            let history_data: Vec<serde_json::Value> = history
                .into_iter()
                .map(|record| {
                    json!({
                        "id": record.id,
                        "user_pubkey": record.user_pubkey,
                        "amount": record.amount,
                        "tier": record.tier,
                        "merit_gained": record.merit_gained,
                        "transaction_signature": record.transaction_signature,
                        "created_at": record.created_at.to_rfc3339()
                    })
                })
                .collect();

            let response = json!({
                "user_pubkey": user_pubkey,
                "history": history_data,
                "count": history_data.len()
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting user donation history: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_user_donation_badges_api(
    State(state): State<AppState>,
    axum::extract::Path(user_pubkey): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get user's donation badges from database
    match get_user_donation_badges(&state.db_pool, &user_pubkey).await {
        Ok(badges) => {
            let badges_data: Vec<serde_json::Value> = badges
                .into_iter()
                .map(|badge| {
                    json!({
                        "tier": badge.tier,
                        "badge_name": badge.badge_name,
                        "earned_at": badge.earned_at.to_rfc3339(),
                        "total_donated": badge.total_donated,
                        "nft_mint": badge.nft_mint
                    })
                })
                .collect();

            let response = json!({
                "user_pubkey": user_pubkey,
                "badges": badges_data
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting user donation badges: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_honor_wall_api(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract query parameters with defaults
    let limit_str = params.get("limit").map(|s| s.as_str()).unwrap_or("50");

    let limit: usize = limit_str.parse().unwrap_or(50).min(1000); // Max 1000 per page

    // Get honor wall data from database
    match get_donation_leaderboard_db(&state.db_pool, limit, 0).await {
        Ok(leaderboard) => {
            let entries: Vec<serde_json::Value> = leaderboard
                .into_iter()
                .map(|entry| {
                    // Determine tier based on total donated amount
                    let tier = if entry.total_donated >= 5.0 {
                        "supreme"
                    } else if entry.total_donated >= 1.0 {
                        "gold"
                    } else if entry.total_donated >= 0.2 {
                        "silver"
                    } else {
                        "bronze"
                    };

                    json!({
                        "rank": entry.rank,
                        "user_pubkey": entry.user_pubkey,
                        "total_donated": entry.total_donated,
                        "tier": tier,
                        "last_donation_at": "2025-01-01T00:00:00Z", // Placeholder
                        "donation_count": 1 // Placeholder
                    })
                })
                .collect();

            let response = json!({
                "entries": entries,
                "count": entries.len()
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting honor wall: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DonationTransactionRequest {
    pub user_pubkey: String,
    pub amount_sol: f64,
    pub tier: String,
    pub transaction_signature: String,
}

async fn submit_donation_transaction_api(
    State(state): State<AppState>,
    axum::extract::Json(request): axum::extract::Json<DonationTransactionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Validate request
    if request.user_pubkey.is_empty() || request.transaction_signature.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Determine merit points based on tier
    let merit_gained = match request.tier.as_str() {
        "bronze" => 65,
        "silver" => 1300,
        "gold" => 14000,
        "supreme" => 120000,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    // Here we would typically:
    // 1. Verify the transaction on Solana
    // 2. Update user donation records
    // 3. Mint NFT badge if applicable
    // 4. Update user state with merit points

    // For now, return success response
    let response = json!({
        "success": true,
        "tier": request.tier,
        "merit_gained": merit_gained,
        "badge_minted": false,
        "nft_mint": null,
        "transaction_signature": request.transaction_signature
    });
    Ok(Json(response))
}

// ===== AMULET API ENDPOINTS =====

async fn get_user_pending_amulets(
    State(state): State<AppState>,
    axum::extract::Path(user_pubkey): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get user's pending amulets from database
    match crate::db::get_user_pending_amulets(&state.db_pool, &user_pubkey).await {
        Ok(pending_amulets) => {
            let amulets_data: Vec<serde_json::Value> = pending_amulets
                .into_iter()
                .map(|amulet| {
                    json!({
                        "id": amulet.id,
                        "user_pubkey": amulet.user_pubkey,
                        "amulet_type": amulet.amulet_type,
                        "source": amulet.source,
                        "created_at": amulet.created_at.to_rfc3339()
                    })
                })
                .collect();

            let response = json!({
                "user_pubkey": user_pubkey,
                "pending_amulets": amulets_data,
                "count": amulets_data.len()
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting user pending amulets: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_user_recent_amulet_drop(
    State(state): State<AppState>,
    axum::extract::Path(user_pubkey): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get user's most recent amulet drop from database
    match crate::db::get_user_pending_amulets(&state.db_pool, &user_pubkey).await {
        Ok(pending_amulets) => {
            // Get the most recent amulet drop (first in the list since ordered by created_at DESC)
            let recent_drop = pending_amulets.into_iter().next();

            let response = if let Some(amulet) = recent_drop {
                json!({
                    "user_pubkey": user_pubkey,
                    "has_recent_drop": true,
                    "recent_drop": {
                        "id": amulet.id,
                        "amulet_type": amulet.amulet_type,
                        "source": amulet.source,
                        "created_at": amulet.created_at.to_rfc3339(),
                        "time_since_drop_seconds": (chrono::Utc::now() - amulet.created_at).num_seconds()
                    }
                })
            } else {
                json!({
                    "user_pubkey": user_pubkey,
                    "has_recent_drop": false,
                    "recent_drop": null
                })
            };
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting user recent amulet drop: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_user_owned_amulets(
    State(state): State<AppState>,
    axum::extract::Path(user_pubkey): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get user's owned amulets from amulet_mint_history table
    match crate::db::get_user_owned_amulets(&state.db_pool, &user_pubkey).await {
        Ok(owned_amulets) => {
            let amulets_data: Vec<serde_json::Value> = owned_amulets
                .into_iter()
                .map(|amulet| {
                    json!({
                        "id": amulet.id,
                        "user_pubkey": amulet.user_pubkey,
                        "amulet_mint": amulet.amulet_mint,
                        "source": amulet.source,
                        "serial_number": amulet.serial_number,
                        "created_at": amulet.created_at.to_rfc3339()
                    })
                })
                .collect();

            let response = json!({
                "user_pubkey": user_pubkey,
                "owned_amulets": amulets_data,
                "count": amulets_data.len()
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting user owned amulets: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
