// src/api_router/profile.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::json;

use super::AppState;
use crate::db::{
    get_user_profile_achievements, get_user_profile_activities, get_user_profile_basic,
    get_user_profile_nfts,
};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/api/profile/{user_pubkey}/basic", get(get_profile_basic))
        .route(
            "/api/profile/{user_pubkey}/activities",
            get(get_profile_activities),
        )
        .route(
            "/api/profile/{user_pubkey}/achievements",
            get(get_profile_achievements),
        )
        .route("/api/profile/{user_pubkey}/nfts", get(get_profile_nfts))
        .with_state(state)
}

/// Get basic user profile data (from user_states table only)
#[utoipa::path(
    get,
    path = "/api/profile/{user_pubkey}/basic",
    params(
        ("user_pubkey" = String, Path, description = "User public key"),
    ),
    responses(
        (status = 200, description = "Basic user profile data", body = serde_json::Value),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Profile"
)]
pub async fn get_profile_basic(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match get_user_profile_basic(&state.db_pool, &user_pubkey).await {
        Ok(profile) => {
            let response = json!({
                "user_pubkey": profile.user_pubkey,
                "merit_points": profile.merit_points,
                "incense_points": profile.incense_points,
                "rank": profile.rank,
                "joined_date": profile.joined_date.map(|d| d.to_rfc3339()),
                "stats": {
                    "total_incense_burned": profile.stats.total_incense_burned,
                    "total_fortunes_drawn": profile.stats.total_fortunes_drawn,
                    "total_wishes_made": profile.stats.total_wishes_made,
                    "total_donated_sol": profile.stats.total_donated_sol
                }
            });

            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting user profile basic: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user activity history
#[utoipa::path(
    get,
    path = "/api/profile/{user_pubkey}/activities",
    params(
        ("user_pubkey" = String, Path, description = "User public key"),
    ),
    responses(
        (status = 200, description = "User activity history", body = serde_json::Value),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Profile"
)]
pub async fn get_profile_activities(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match get_user_profile_activities(&state.db_pool, &user_pubkey).await {
        Ok(activities) => {
            let response = json!({
                "activities": activities.into_iter().map(|activity| {
                    json!({
                        "activity_type": activity.activity_type,
                        "description": activity.description,
                        "merit_gained": activity.merit_gained,
                        "created_at": activity.created_at.to_rfc3339()
                    })
                }).collect::<Vec<_>>()
            });

            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting user profile activities: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user achievements
#[utoipa::path(
    get,
    path = "/api/profile/{user_pubkey}/achievements",
    params(
        ("user_pubkey" = String, Path, description = "User public key"),
    ),
    responses(
        (status = 200, description = "User achievements", body = serde_json::Value),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Profile"
)]
pub async fn get_profile_achievements(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match get_user_profile_achievements(&state.db_pool, &user_pubkey).await {
        Ok(achievements) => {
            let response = json!({
                "achievements": achievements.into_iter().map(|achievement| {
                    json!({
                        "title": achievement.title,
                        "description": achievement.description,
                        "unlocked": achievement.unlocked,
                        "unlocked_at": achievement.unlocked_at.map(|d| d.to_rfc3339())
                    })
                }).collect::<Vec<_>>()
            });

            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting user profile achievements: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user NFT statistics
#[utoipa::path(
    get,
    path = "/api/profile/{user_pubkey}/nfts",
    params(
        ("user_pubkey" = String, Path, description = "User public key"),
    ),
    responses(
        (status = 200, description = "User NFT statistics", body = serde_json::Value),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error"),
    ),
    tag = "Profile"
)]
pub async fn get_profile_nfts(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match get_user_profile_nfts(&state.db_pool, &user_pubkey).await {
        Ok(nfts) => {
            let response = json!({
                "nfts": {
                    "amulet_count": nfts.amulet_count,
                    "fortune_nft_count": nfts.fortune_nft_count,
                    "buddha_nft_count": nfts.buddha_nft_count
                }
            });

            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error getting user profile NFTs: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
