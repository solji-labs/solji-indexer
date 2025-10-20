// src/api/wishes.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::collections::HashMap;

use super::AppState;
use crate::db::{
    get_public_wishes as db_get_public_wishes,
    get_user_daily_wish_count as db_get_user_daily_wish_count,
    get_user_wish_tower_stats as db_get_user_wish_tower_stats,
    get_user_wishes as db_get_user_wishes, get_wishes as db_get_wishes, like_wish_by_id,
};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/api/wishes", get(get_wishes))
        .route("/api/wishes/public", get(get_public_wishes))
        .route("/api/wishes/user/{user_pubkey}", get(get_user_wishes))
        .route(
            "/api/wishes/user/{user_pubkey}/count",
            get(get_user_daily_count),
        )
        .route("/api/wish-tower/{user_pubkey}", get(get_user_tower))
        .route("/api/wishes/{wish_id}/like", post(like_wish))
        .with_state(state)
}

/// Get paginated wishes
///
/// Returns a paginated list of all wishes in the system
#[utoipa::path(
    get,
    path = "/api/wishes",
    params(
        ("limit" = Option<i32>, Query, description = "Number of wishes to return (max 100)", example = 20),
        ("offset" = Option<i32>, Query, description = "Offset for pagination", example = 0),
    ),
    responses(
        (status = 200, description = "List of wishes", body = serde_json::Value,
            example = json!({
                "wishes": [
                    {
                        "id": 1,
                        "wish_id": 12345,
                        "user_pubkey": "5xot9PdcigoDgdXJYuSGKmHBhcQn3WHPh1EwLyBNxmNw",
                        "content": "May all beings be happy",
                        "likes": 42,
                        "created_at": "2025-01-01T00:00:00Z",
                        "updated_at": "2025-01-01T00:00:00Z"
                    }
                ],
                "pagination": {
                    "limit": 20,
                    "offset": 0,
                    "count": 1
                }
            })
        ),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wishes"
)]
pub async fn get_wishes(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit: i32 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(20)
        .min(100);
    let offset: i32 = params
        .get("offset")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
        .max(0);

    match db_get_wishes(&state.db_pool, limit, offset).await {
        Ok(wishes) => {
            let wishes_data: Vec<_> = wishes
                .into_iter()
                .map(|w| {
                    json!({
                        "id": w.id,
                        "wish_id": w.wish_id,
                        "user_pubkey": w.user_pubkey,
                        "content": w.content,
                        "likes": w.likes,
                        "created_at": w.created_at.to_rfc3339(),
                        "updated_at": w.updated_at.to_rfc3339()
                    })
                })
                .collect();

            Ok(Json(json!({
                "wishes": wishes_data,
                "pagination": {
                    "limit": limit,
                    "offset": offset,
                    "count": wishes_data.len()
                }
            })))
        }
        Err(e) => {
            eprintln!("Database error getting wishes: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get public wishes
///
/// Returns a paginated list of public wishes visible to all users
#[utoipa::path(
    get,
    path = "/api/wishes/public",
    params(
        ("limit" = Option<i32>, Query, description = "Number of wishes (max 100)", example = 20),
        ("offset" = Option<i32>, Query, description = "Offset for pagination", example = 0),
    ),
    responses(
        (status = 200, description = "Public wishes", body = serde_json::Value,
            example = json!({
                "wishes": [
                    {
                        "id": 1,
                        "wish_id": 12345,
                        "user_pubkey": "5xot9PdcigoDgdXJYuSGKmHBhcQn3WHPh1EwLyBNxmNw",
                        "content": "Public wish content",
                        "likes": 10,
                        "created_at": "2025-01-01T00:00:00Z"
                    }
                ],
                "pagination": {
                    "limit": 20,
                    "offset": 0,
                    "count": 1
                }
            })
        ),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wishes"
)]
pub async fn get_public_wishes(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit: i32 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(20)
        .min(100);
    let offset: i32 = params
        .get("offset")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
        .max(0);

    match db_get_public_wishes(&state.db_pool, limit, offset).await {
        Ok(wishes) => {
            let wishes_data: Vec<_> = wishes
                .into_iter()
                .map(|w| {
                    json!({
                        "id": w.id,
                        "wish_id": w.wish_id,
                        "user_pubkey": w.user_pubkey,
                        "content": w.content,
                        "likes": w.likes,
                        "created_at": w.created_at.to_rfc3339()
                    })
                })
                .collect();

            Ok(Json(json!({
                "wishes": wishes_data,
                "pagination": {
                    "limit": limit,
                    "offset": offset,
                    "count": wishes_data.len()
                }
            })))
        }
        Err(e) => {
            eprintln!("Database error getting public wishes: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user wishes
///
/// Returns all wishes created by a specific user
#[utoipa::path(
    get,
    path = "/api/wishes/user/{user_pubkey}",
    params(
        ("user_pubkey" = String, Path, description = "User public key", example = "5xot9PdcigoDgdXJYuSGKmHBhcQn3WHPh1EwLyBNxmNw"),
        ("limit" = Option<i32>, Query, description = "Number of wishes", example = 20),
        ("offset" = Option<i32>, Query, description = "Offset", example = 0),
    ),
    responses(
        (status = 200, description = "User wishes", body = serde_json::Value,
            example = json!({
                "user_pubkey": "5xot9PdcigoDgdXJYuSGKmHBhcQn3WHPh1EwLyBNxmNw",
                "wishes": [],
                "pagination": {
                    "limit": 20,
                    "offset": 0,
                    "count": 0
                }
            })
        ),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wishes"
)]
pub async fn get_user_wishes(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit: i32 = params
        .get("limit")
        .and_then(|s| s.parse().ok())
        .unwrap_or(20)
        .min(100);
    let offset: i32 = params
        .get("offset")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
        .max(0);

    match db_get_user_wishes(&state.db_pool, &user_pubkey, limit, offset).await {
        Ok(wishes) => {
            let wishes_data: Vec<_> = wishes
                .into_iter()
                .map(|w| {
                    json!({
                        "id": w.id,
                        "wish_id": w.wish_id,
                        "user_pubkey": w.user_pubkey,
                        "content": w.content,
                        "likes": w.likes,
                        "created_at": w.created_at.to_rfc3339()
                    })
                })
                .collect();

            Ok(Json(json!({
                "user_pubkey": user_pubkey,
                "wishes": wishes_data,
                "pagination": {
                    "limit": limit,
                    "offset": offset,
                    "count": wishes_data.len()
                }
            })))
        }
        Err(e) => {
            eprintln!("Database error getting user wishes: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user daily wish count
///
/// Returns the number of wishes a user has made today (for rate limiting)
#[utoipa::path(
    get,
    path = "/api/wishes/user/{user_pubkey}/count",
    params(
        ("user_pubkey" = String, Path, description = "User public key", example = "5xot9PdcigoDgdXJYuSGKmHBhcQn3WHPh1EwLyBNxmNw"),
    ),
    responses(
        (status = 200, description = "Daily wish count", body = serde_json::Value,
            example = json!({
                "user_pubkey": "5xot9PdcigoDgdXJYuSGKmHBhcQn3WHPh1EwLyBNxmNw",
                "daily_wish_count": 2,
                "max_daily_limit": 3
            })
        ),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wishes"
)]
pub async fn get_user_daily_count(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match db_get_user_daily_wish_count(&state.db_pool, &user_pubkey).await {
        Ok(count) => Ok(Json(json!({
            "user_pubkey": user_pubkey,
            "daily_wish_count": count,
            "max_daily_limit": 3
        }))),
        Err(e) => {
            eprintln!("Database error getting user daily wish count: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user wish tower stats
///
/// Returns the user's wish tower statistics including total wishes and level
#[utoipa::path(
    get,
    path = "/api/wish-tower/{user_pubkey}",
    params(
        ("user_pubkey" = String, Path, description = "User public key", example = "5xot9PdcigoDgdXJYuSGKmHBhcQn3WHPh1EwLyBNxmNw"),
    ),
    responses(
        (status = 200, description = "Wish tower statistics", body = serde_json::Value,
            example = json!({
                "user_pubkey": "5xot9PdcigoDgdXJYuSGKmHBhcQn3WHPh1EwLyBNxmNw",
                "total_wishes": 10,
                "level": 2,
                "last_updated": "2025-01-01T00:00:00Z"
            })
        ),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wishes"
)]
pub async fn get_user_tower(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match db_get_user_wish_tower_stats(&state.db_pool, &user_pubkey).await {
        Ok(stats) => Ok(Json(json!({
            "user_pubkey": stats.user_pubkey,
            "total_wishes": stats.total_wishes,
            "level": stats.level,
            "last_updated": stats.last_updated.map(|dt| dt.to_rfc3339())
        }))),
        Err(e) => {
            eprintln!("Database error getting user wish tower: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Like a wish
///
/// Increments the like count for a specific wish
#[utoipa::path(
    post,
    path = "/api/wishes/{wish_id}/like",
    params(
        ("wish_id" = i64, Path, description = "Wish ID to like", example = 12345),
    ),
    responses(
        (status = 200, description = "Wish liked successfully", body = serde_json::Value,
            example = json!({
                "wish_id": 12345,
                "likes": 43,
                "success": true
            })
        ),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wishes"
)]
pub async fn like_wish(
    State(state): State<AppState>,
    Path(wish_id): Path<i64>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match like_wish_by_id(&state.db_pool, wish_id).await {
        Ok(new_likes) => Ok(Json(json!({
            "wish_id": wish_id,
            "likes": new_likes,
            "success": true
        }))),
        Err(e) => {
            eprintln!("Database error liking wish: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
