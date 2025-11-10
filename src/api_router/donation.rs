// src/api/donation.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::collections::HashMap;

use super::{models::DonationTierInfo, AppState};
use crate::db::{
    check_user_in_top_10000_donors, get_donation_leaderboard, get_donation_leaderboard_by_period,
    get_user_donation_badges, get_user_donation_history,
};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/api/donation/tiers", get(get_tiers))
        .route("/api/donation/leaderboard", get(get_leaderboard))
        .route("/api/donation/check-top-10000", get(check_top_donors))
        .route(
            "/api/donation/user/{user_pubkey}/history",
            get(get_user_history),
        )
        .route(
            "/api/donation/user/{user_pubkey}/badges",
            get(get_user_badges),
        )
        .route("/api/donation/honor-wall", get(get_honor_wall))
        .route("/api/donation/submit", post(submit_transaction))
        .with_state(state)
}

/// Get donation tiers
#[utoipa::path(
    get,
    path = "/api/donation/tiers",
    responses(
        (status = 200, description = "Donation tiers", body = serde_json::Value)
    ),
    tag = "Donation"
)]
pub async fn get_tiers() -> Result<Json<serde_json::Value>, StatusCode> {
    let tiers = vec![
        DonationTierInfo {
            tier: "bronze".to_string(),
            name: "Bronze Devotee".to_string(),
            name_en: "Bronze Devotee".to_string(),
            min_amount: 0.05,
            merit_points: 65,
            badge: "Bronze Merit Badge NFT".to_string(),
            benefits: vec!["Display name on incense wall".to_string()],
        },
        DonationTierInfo {
            tier: "silver".to_string(),
            name: "Silver Layman".to_string(),
            name_en: "Silver Layman".to_string(),
            min_amount: 0.2,
            merit_points: 1300,
            badge: "Silver Progress Badge NFT".to_string(),
            benefits: vec!["Vote on temple proposals".to_string()],
        },
        DonationTierInfo {
            tier: "gold".to_string(),
            name: "Gold Guardian".to_string(),
            name_en: "Gold Guardian".to_string(),
            min_amount: 1.0,
            merit_points: 14000,
            badge: "Gold Guardian Badge NFT".to_string(),
            benefits: vec!["Participate in temple NFT governance".to_string()],
        },
        DonationTierInfo {
            tier: "supreme".to_string(),
            name: "Supreme Patron".to_string(),
            name_en: "Supreme Patron".to_string(),
            min_amount: 5.0,
            merit_points: 120000,
            badge: "Supreme Dragon Badge NFT".to_string(),
            benefits: vec!["Unlock easter eggs + temple co-builder status".to_string()],
        },
    ];

    Ok(Json(json!({ "tiers": tiers })))
}

/// Get donation leaderboard
#[utoipa::path(
    get,
    path = "/api/donation/leaderboard",
    params(
        ("period" = Option<String>, Query, description = "Time period: 'all', 'daily', 'weekly', 'monthly'. Defaults to 'all' if not provided"),
        ("limit" = Option<usize>, Query, description = "Number of entries (max 1000)"),
        ("offset" = Option<usize>, Query, description = "Offset"),
    ),
    responses(
        (status = 200, description = "Donation leaderboard", body = serde_json::Value)
    ),
    tag = "Donation"
)]
pub async fn get_leaderboard(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Support both explicit period parameter and default behavior
    let period = params.get("period").map(|s| s.as_str()).unwrap_or("all");
    let limit: usize = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100)
        .min(1000);
    let offset: usize = params
        .get("offset")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // Validate period if provided
    let valid_periods = ["all", "daily", "weekly", "monthly"];
    if !valid_periods.contains(&period) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let leaderboard = if period == "all" {
        // Use cumulative data from user_donations table
        get_donation_leaderboard(&state.db_pool, limit, offset).await
    } else {
        // Aggregate data from donation_history table by period
        get_donation_leaderboard_by_period(&state.db_pool, period, limit, offset).await
    };

    match leaderboard {
        Ok(leaderboard) => {
            let data: Vec<_> = leaderboard
                .into_iter()
                .map(|e| {
                    json!({
                        "rank": e.rank,
                        "user_pubkey": e.user_pubkey,
                        "total_donated": e.total_donated
                    })
                })
                .collect();

            Ok(Json(json!({
                "leaderboard": data,
                "period": period,  // Include period in response for clarity
                "pagination": { "limit": limit, "offset": offset, "count": data.len() }
            })))
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Check if user is in top 10000 donors
#[utoipa::path(
    get,
    path = "/api/donation/check-top-10000",
    params(
        ("user" = String, Query, description = "User public key"),
    ),
    responses(
        (status = 200, description = "Top donor status", body = serde_json::Value)
    ),
    tag = "Donation"
)]
pub async fn check_top_donors(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_pubkey = params.get("user").ok_or(StatusCode::BAD_REQUEST)?;

    match check_user_in_top_10000_donors(&state.db_pool, user_pubkey).await {
        Ok(is_in_top) => Ok(Json(json!({
            "user_pubkey": user_pubkey,
            "is_in_top_10000": is_in_top
        }))),
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user donation history
#[utoipa::path(
    get,
    path = "/api/donation/user/{user_pubkey}/history",
    params(
        ("user_pubkey" = String, Path, description = "User public key"),
        ("limit" = Option<i32>, Query, description = "Number of records"),
    ),
    responses(
        (status = 200, description = "Donation history", body = serde_json::Value)
    ),
    tag = "Donation"
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

    match get_user_donation_history(&state.db_pool, &user_pubkey, limit).await {
        Ok(history) => {
            let data: Vec<_> = history
                .into_iter()
                .map(|r| {
                    json!({
                        "id": r.id,
                        "amount": r.amount,
                        "tier": r.tier,
                        "merit_gained": r.merit_gained,
                        "transaction_signature": r.transaction_signature,
                        "created_at": r.created_at.to_rfc3339()
                    })
                })
                .collect();

            Ok(Json(json!({
                "user_pubkey": user_pubkey,
                "history": data,
                "count": data.len()
            })))
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user donation badges
#[utoipa::path(
    get,
    path = "/api/donation/user/{user_pubkey}/badges",
    params(
        ("user_pubkey" = String, Path, description = "User public key"),
    ),
    responses(
        (status = 200, description = "User badges", body = serde_json::Value)
    ),
    tag = "Donation"
)]
pub async fn get_user_badges(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match get_user_donation_badges(&state.db_pool, &user_pubkey).await {
        Ok(badges) => {
            let data: Vec<_> = badges
                .into_iter()
                .map(|b| {
                    json!({
                        "tier": b.tier,
                        "badge_name": b.badge_name,
                        "earned_at": b.earned_at.to_rfc3339(),
                        "total_donated": b.total_donated,
                        "nft_mint": b.nft_mint
                    })
                })
                .collect();

            Ok(Json(json!({
                "user_pubkey": user_pubkey,
                "badges": data
            })))
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get honor wall
#[utoipa::path(
    get,
    path = "/api/donation/honor-wall",
    params(
        ("limit" = Option<usize>, Query, description = "Number of entries"),
    ),
    responses(
        (status = 200, description = "Honor wall", body = serde_json::Value)
    ),
    tag = "Donation"
)]
pub async fn get_honor_wall(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit: usize = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(50)
        .min(1000);

    match get_donation_leaderboard(&state.db_pool, limit, 0).await {
        Ok(leaderboard) => {
            let entries: Vec<_> = leaderboard
                .into_iter()
                .map(|e| {
                    let tier = if e.total_donated >= 5.0 {
                        "supreme"
                    } else if e.total_donated >= 1.0 {
                        "gold"
                    } else if e.total_donated >= 0.2 {
                        "silver"
                    } else {
                        "bronze"
                    };

                    json!({
                        "rank": e.rank,
                        "user_pubkey": e.user_pubkey,
                        "total_donated": e.total_donated,
                        "tier": tier
                    })
                })
                .collect();

            Ok(Json(json!({
                "entries": entries,
                "count": entries.len()
            })))
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Submit donation transaction
#[utoipa::path(
    post,
    path = "/api/donation/submit",
    request_body = super::models::DonationTransactionRequest,
    responses(
        (status = 200, description = "Transaction submitted", body = serde_json::Value)
    ),
    tag = "Donation"
)]
pub async fn submit_transaction(
    State(state): State<AppState>,
    Json(request): Json<super::models::DonationTransactionRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if request.user_pubkey.is_empty() || request.transaction_signature.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let merit_gained = match request.tier.as_str() {
        "bronze" => 65,
        "silver" => 1300,
        "gold" => 14000,
        "supreme" => 120000,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    Ok(Json(json!({
        "success": true,
        "tier": request.tier,
        "merit_gained": merit_gained,
        "transaction_signature": request.transaction_signature
    })))
}
