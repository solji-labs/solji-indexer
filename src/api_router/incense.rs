// api/incense.rs

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::json;
use std::collections::HashMap;

use super::{models::IncenseTypeInfo, AppState};
use crate::db::{
    check_daily_incense_limit, get_parsed_incense_leaderboard_by_period,
    get_user_incense_burn_count, get_user_incense_burn_history, get_user_incense_nfts,
};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/api/incense/types", get(get_incense_types))
        .route("/api/incense/can-burn", get(check_can_burn_incense))
        .route(
            "/api/incense/user/{user_pubkey}/burn-count",
            get(get_user_burn_count),
        )
        .route("/api/incense/user/{user_pubkey}/nfts", get(get_user_nfts))
        .route(
            "/api/incense/user/{user_pubkey}/history",
            get(get_user_history),
        )
        .route("/api/incense/leaderboard", get(get_leaderboard))
        .with_state(state)
}

/// Get all incense types
///
/// Returns information about all available incense types including prices and merit points
#[utoipa::path(
    get,
    path = "/api/incense/types",
    responses(
        (status = 200, description = "List of incense types", body = serde_json::Value,
            example = json!({
                "incense_types": [
                    {
                        "id": "basic",
                        "name": "清香",
                        "name_en": "Basic Incense",
                        "price": 0.01,
                        "merit_points": 1,
                        "description": "Simple and pure, for daily devotion",
                        "image": "/traditional-incense-stick-glowing.jpg",
                        "daily_limit": 10
                    }
                ]
            })
        )
    ),
    tag = "Incense"
)]
pub async fn get_incense_types() -> Result<Json<serde_json::Value>, StatusCode> {
    let incense_types = vec![
        IncenseTypeInfo {
            id: "basic".to_string(),
            name: "清香".to_string(),
            name_en: "Clear Incense".to_string(),
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
            name_en: "Ambergris Incense".to_string(),
            price: 0.1,
            merit_points: 1200,
            description: "Rare dragon incense for great fortune".to_string(),
            image: "/mystical-dragon-incense-with-purple-smoke.jpg".to_string(),
            daily_limit: 10,
        },
        IncenseTypeInfo {
            id: "supreme".to_string(),
            name: "至尊香".to_string(),
            name_en: "Supreme Spirit Incense".to_string(),
            price: 0.3,
            merit_points: 3400,
            description: "The ultimate offering for enlightenment".to_string(),
            image: "/supreme-golden-incense-with-rainbow-aura.jpg".to_string(),
            daily_limit: 10,
        },
    ];

    Ok(Json(json!({ "incense_types": incense_types })))
}

/// Check if user can burn incense
///
/// Validates whether a user can burn a specific amount of incense based on daily limits
#[utoipa::path(
    get,
    path = "/api/incense/can-burn",
    params(
        ("user" = String, Query, description = "User public key"),
        ("incense_type" = i32, Query, description = "Incense type ID"),
        ("amount" = i32, Query, description = "Amount to burn"),
    ),
    responses(
        (status = 200, description = "Check result", body = serde_json::Value,
            example = json!({
                "can_burn": true,
                "user": "5xot9PdcigoDgdXJYuSGKmHBhcQn3WHPh1EwLyBNxmNw",
                "incense_type": 1,
                "requested_amount": 5,
                "max_daily_limit": 10
            })
        ),
        (status = 400, description = "Invalid parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Incense"
)]
pub async fn check_can_burn_incense(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_pubkey = params.get("user").ok_or(StatusCode::BAD_REQUEST)?;
    let incense_type_str = params.get("incense_type").ok_or(StatusCode::BAD_REQUEST)?;
    let amount_str = params.get("amount").ok_or(StatusCode::BAD_REQUEST)?;

    let incense_type: i32 = incense_type_str
        .parse()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let amount: i32 = amount_str.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    match check_daily_incense_limit(&state.db_pool, user_pubkey, incense_type, amount).await {
        Ok(can_burn) => Ok(Json(json!({
            "can_burn": can_burn,
            "user": user_pubkey,
            "incense_type": incense_type,
            "requested_amount": amount,
            "max_daily_limit": 10
        }))),
        Err(e) => {
            eprintln!("Database error checking incense limit: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user incense burn count
///
/// Returns the daily burn count for each incense type for a specific user
#[utoipa::path(
    get,
    path = "/api/incense/user/{user_pubkey}/burn-count",
    params(
        ("user_pubkey" = String, Path, description = "User public key"),
    ),
    responses(
        (status = 200, description = "User burn counts", body = serde_json::Value,
            example = json!({
                "user_pubkey": "5xot9PdcigoDgdXJYuSGKmHBhcQn3WHPh1EwLyBNxmNw",
                "burn_counts": {
                    "0": 5,
                    "1": 3
                },
                "max_daily_limit": 10
            })
        ),
        (status = 500, description = "Internal server error")
    ),
    tag = "Incense"
)]
pub async fn get_user_burn_count(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match get_user_incense_burn_count(&state.db_pool, &user_pubkey).await {
        Ok(burn_counts) => Ok(Json(json!({
            "user_pubkey": user_pubkey,
            "burn_counts": burn_counts,
            "max_daily_limit": 10
        }))),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user incense NFTs
///
/// Returns all incense NFTs owned by a specific user
#[utoipa::path(
    get,
    path = "/api/incense/user/{user_pubkey}/nfts",
    params(
        ("user_pubkey" = String, Path, description = "User public key"),
    ),
    responses(
        (status = 200, description = "User incense NFTs", body = serde_json::Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "Incense"
)]
pub async fn get_user_nfts(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match get_user_incense_nfts(&state.db_pool, &user_pubkey).await {
        Ok(nfts) => Ok(Json(json!({
            "user_pubkey": user_pubkey,
            "nfts": nfts
        }))),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user incense burn history
///
/// Returns the burn history for a specific user with pagination
#[utoipa::path(
    get,
    path = "/api/incense/user/{user_pubkey}/history",
    params(
        ("user_pubkey" = String, Path, description = "User public key"),
        ("limit" = Option<i32>, Query, description = "Number of records to return (max 100)"),
    ),
    responses(
        (status = 200, description = "User burn history", body = serde_json::Value),
        (status = 500, description = "Internal server error")
    ),
    tag = "Incense"
)]
pub async fn get_user_history(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit: i32 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(20)
        .min(100);

    match get_user_incense_burn_history(&state.db_pool, &user_pubkey, limit).await {
        Ok(history) => {
            let history_data: Vec<_> = history
                .into_iter()
                .map(|r| {
                    json!({
                        "id": r.id,
                        "incense_type": r.incense_type,
                        "incense_amount": r.incense_amount,
                        "merit_gained": r.merit_gained,
                        "incense_points_gained": r.incense_points_gained,
                        "transaction_signature": r.transaction_signature,
                        "created_at": r.created_at.to_rfc3339()
                    })
                })
                .collect();

            Ok(Json(json!({
                "user_pubkey": user_pubkey,
                "history": history_data,
                "count": history_data.len()
            })))
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get incense leaderboard
///
/// Returns the incense burning leaderboard for a specific time period
#[utoipa::path(
    get,
    path = "/api/incense/leaderboard",
    params(
        ("period" = Option<String>, Query, description = "Time period: all, daily, weekly, monthly"),
    ),
    responses(
        (status = 200, description = "Incense leaderboard", body = serde_json::Value,
            example = json!({
                "period": "all",
                "leaderboard": [
                    {
                        "rank": 1,
                        "user_pubkey": "5xot9PdcigoDgdXJYuSGKmHBhcQn3WHPh1EwLyBNxmNw",
                        "total_incense_points": 1000,
                        "burn_count": 50
                    }
                ],
                "count": 1
            })
        ),
        (status = 400, description = "Invalid period parameter"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Incense"
)]
pub async fn get_leaderboard(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let period = params.get("period").map(|s| s.as_str()).unwrap_or("all");

    if !["all", "daily", "weekly", "monthly"].contains(&period) {
        return Err(StatusCode::BAD_REQUEST);
    }

    match get_parsed_incense_leaderboard_by_period(&state.db_pool, period).await {
        Ok(leaderboard) => {
            let data: Vec<_> = leaderboard
                .into_iter()
                .map(|e| {
                    json!({
                        "rank": e.rank,
                        "user_pubkey": e.user_pubkey,
                        "total_incense_points": e.total_incense_points,
                        "burn_count": e.burn_count
                    })
                })
                .collect();

            Ok(Json(json!({
                "period": period,
                "leaderboard": data,
                "count": data.len()
            })))
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
