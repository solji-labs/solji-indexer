// src/api/wishes.rs
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::collections::HashMap;

use super::AppState;
use crate::db::{
    check_user_liked_wish, get_public_wishes as db_get_public_wishes,
    get_public_wishes_with_user_likes, get_user_daily_wish_count as db_get_user_daily_wish_count,
    get_user_wish_tower_stats as db_get_user_wish_tower_stats,
    get_user_wishes as db_get_user_wishes, get_user_wishes_with_likes, get_wishes as db_get_wishes,
    get_wishes_with_user_likes, insert_wish_like, like_wish_by_id,
};

/// Create wish request
#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
pub struct CreateWishRequest {
    /// IPFS CID of the wish content
    pub cid: String,
    /// Whether the wish should be anonymous
    pub is_anonymous: bool,
}

/// Convert IPFS CID to 32-byte array for contract
fn cid_to_content_hash(cid: &str) -> Result<[u8; 32], String> {
    // For IPFS CID v0 (starts with "Qm"), extract the 32-byte hash
    if cid.starts_with("Qm") && cid.len() == 46 {
        // CID v0 format: base58 encoded, first 34 bytes are multihash
        // We need to decode and extract the 32-byte SHA-256 hash
        match bs58::decode(cid).into_vec() {
            Ok(decoded) => {
                if decoded.len() >= 34 {
                    // Skip first 2 bytes (multihash header) and take 32 bytes
                    let mut hash = [0u8; 32];
                    hash.copy_from_slice(&decoded[2..34]);
                    Ok(hash)
                } else {
                    Err("Invalid CID length".to_string())
                }
            }
            Err(e) => Err(format!("Failed to decode CID: {}", e)),
        }
    } else {
        Err("Only CID v0 (Qm...) is currently supported".to_string())
    }
}

// Helper function to retrieve IPFS content for wishes with like status from pairs
async fn get_wishes_with_ipfs_content_and_likes_from_pairs(
    wish_likes: Vec<(crate::db::models::Wish, bool)>,
    state: &AppState,
) -> Vec<serde_json::Value> {
    let wishes: Vec<crate::db::models::Wish> = wish_likes.iter().map(|(w, _)| w.clone()).collect();

    // Extract IPFS hashes for batch retrieval - convert hex to CID
    let ipfs_hashes: Vec<String> = wishes
        .iter()
        .filter_map(|w| {
            // If content is hex-encoded 32-byte hash (64 characters), convert to CID
            if w.content.len() == 64 {
                match hex::decode(&w.content) {
                    Ok(bytes) if bytes.len() == 32 => {
                        // Convert 32-byte hash back to CID v0 format
                        let mut cid_bytes = vec![0x12, 0x20]; // multihash header for sha2-256
                        cid_bytes.extend_from_slice(&bytes);
                        let cid = bs58::encode(&cid_bytes).into_string();
                        Some(cid)
                    }
                    _ => None,
                }
            }
            // If content is already a CID (Qm...), use it directly
            else if !w.content.is_empty() && w.content.starts_with("Qm") {
                Some(w.content.clone())
            } else {
                None
            }
        })
        .collect();

    // Get content from IPFS if we have hashes - inline batch logic
    let ipfs_contents = if !ipfs_hashes.is_empty() {
        // Create futures for concurrent requests using ureq
        let futures = ipfs_hashes.into_iter().map(|hash| {
            let gateway_url = format!("{}/{}", state.config.pinata_gateway, hash);
            async move {
                let hash_for_result = hash.clone();
                let gateway_url_for_result = gateway_url.clone();

                // Use spawn_blocking for ureq synchronous calls
                match tokio::task::spawn_blocking(move || {
                    let agent = ureq::Agent::new();

                    match agent
                        .get(&gateway_url)
                        .timeout(std::time::Duration::from_secs(30))
                        .call()
                    {
                        Ok(resp) => {
                            if resp.status() == 200 {
                                // Get content type from headers
                                let content_type = resp
                                    .header("content-type")
                                    .unwrap_or("application/octet-stream")
                                    .to_string();

                                // Get response body
                                use std::io::Read;
                                let mut reader = resp.into_reader();
                                let mut bytes = Vec::new();
                                match reader.read_to_end(&mut bytes) {
                                    Ok(_) => Ok((content_type, bytes)),
                                    Err(e) => {
                                        println!(
                                            " [IPFS] Failed to read body for {}: {:?}",
                                            hash, e
                                        );
                                        Err((hash, format!("Body read error: {:?}", e)))
                                    }
                                }
                            } else {
                                println!(" [IPFS] Gateway returned {} for {}", resp.status(), hash);
                                Err((hash, format!("HTTP {}", resp.status())))
                            }
                        }
                        Err(e) => {
                            println!(" [IPFS] Failed to fetch {}: {:?}", hash, e);
                            Err((hash, format!("Network error: {:?}", e)))
                        }
                    }
                })
                .await
                {
                    Ok(Ok((content_type, content_bytes))) => {
                        // Parse content
                        let content = if content_type.contains("application/json") {
                            match serde_json::from_slice(&content_bytes) {
                                Ok(json_value) => json_value,
                                Err(_) => serde_json::Value::String(
                                    String::from_utf8_lossy(&content_bytes).to_string(),
                                ),
                            }
                        } else {
                            serde_json::Value::String(
                                String::from_utf8_lossy(&content_bytes).to_string(),
                            )
                        };

                        println!(
                            " [IPFS] Retrieved {} ({} bytes)",
                            hash_for_result,
                            content_bytes.len()
                        );

                        Ok(json!({
                            "hash": hash_for_result,
                            "content": content,
                            "content_type": content_type,
                            "size": content_bytes.len(),
                            "gateway_url": gateway_url_for_result
                        }))
                    }
                    Ok(Err((hash, error))) => Err((hash, error)),
                    Err(e) => {
                        println!(" [IPFS] Task join error for {}: {:?}", hash_for_result, e);
                        Err((hash_for_result, format!("Task error: {:?}", e)))
                    }
                }
            }
        });

        // Execute all requests concurrently
        match futures::future::join_all(futures)
            .await
            .into_iter()
            .filter_map(|result| match result {
                Ok(content) => {
                    if let (Some(hash), Some(content_val)) = (
                        content.get("hash").and_then(|h| h.as_str()),
                        content.get("content"),
                    ) {
                        Some((hash.to_string(), content_val.clone()))
                    } else {
                        None
                    }
                }
                Err(_) => None,
            })
            .collect::<std::collections::HashMap<String, serde_json::Value>>()
        {
            contents => contents,
        }
    } else {
        std::collections::HashMap::new()
    };

    // Create a mapping from original content to CID for lookup
    let mut content_to_cid = std::collections::HashMap::new();
    for w in &wishes {
        let cid = if !w.content.is_empty() && w.content.starts_with("Qm") {
            w.content.clone()
        } else if w.content.len() == 64 {
            match hex::decode(&w.content) {
                Ok(bytes) if bytes.len() == 32 => {
                    let mut cid_bytes = vec![0x12, 0x20];
                    cid_bytes.extend_from_slice(&bytes);
                    bs58::encode(&cid_bytes).into_string()
                }
                _ => w.content.clone(),
            }
        } else {
            w.content.clone()
        };
        content_to_cid.insert(w.content.clone(), cid);
    }

    // Map wishes with like status to JSON with IPFS content
    wish_likes
        .into_iter()
        .map(|(w, is_liked)| {
            // Get the CID for this wish's content
            let cid = content_to_cid.get(&w.content).unwrap_or(&w.content).clone();

            // Get content from IPFS using CID as key, or fallback to hash value
            let content = if let Some(ipfs_content) = ipfs_contents.get(&cid) {
                ipfs_content.clone()
            } else {
                // If IPFS request failed, return the original hash string
                serde_json::Value::String(w.content.clone())
            };

            json!({
                "id": w.id,
                "wish_id": w.wish_id,
                "user_pubkey": w.user_pubkey,
                "content": content,
                "likes": w.likes,
                "is_liked": is_liked,
                "created_at": w.created_at.to_rfc3339()
            })
        })
        .collect()
}

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
                        "is_liked": false,
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
pub async fn get_wishes(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    headers: HeaderMap,
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
    let user_pubkey = headers
        .get("x-user-pubkey")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    match get_wishes_with_user_likes(&state.db_pool, user_pubkey.as_deref(), limit, offset).await {
        Ok(wish_likes) => {
            let wishes_data =
                get_wishes_with_ipfs_content_and_likes_from_pairs(wish_likes, &state).await;

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
                        "is_liked": false,
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
    headers: HeaderMap,
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
    let user_pubkey = headers
        .get("x-user-pubkey")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    match get_public_wishes_with_user_likes(&state.db_pool, user_pubkey.as_deref(), limit, offset)
        .await
    {
        Ok(wish_likes) => {
            let wishes_data =
                get_wishes_with_ipfs_content_and_likes_from_pairs(wish_likes, &state).await;

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
    headers: HeaderMap,
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
    let viewer_pubkey = headers
        .get("x-user-pubkey")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    match get_user_wishes_with_likes(&state.db_pool, &user_pubkey, limit, offset).await {
        Ok(wish_likes) => {
            let wishes_data =
                get_wishes_with_ipfs_content_and_likes_from_pairs(wish_likes, &state).await;

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
/// Records user like for a specific wish and increments the like count
#[utoipa::path(
    post,
    path = "/api/wishes/{wish_id}/like",
    params(
        ("wish_id" = i64, Path, description = "Wish ID to like", example = 12345),
    ),
    request_body(content = serde_json::Value, description = "Request headers should include x-user-pubkey"),
    responses(
        (status = 200, description = "Wish liked successfully", body = serde_json::Value,
            example = json!({
                "wish_id": 12345,
                "likes": 43,
                "success": true
            })
        ),
        (status = 400, description = "Bad request - missing x-user-pubkey header"),
        (status = 409, description = "Already liked this wish"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Wishes"
)]
pub async fn like_wish(
    State(state): State<AppState>,
    Path(wish_id): Path<i64>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_pubkey = match headers.get("x-user-pubkey") {
        Some(header_value) => match header_value.to_str() {
            Ok(pubkey) => pubkey,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        },
        None => return Err(StatusCode::BAD_REQUEST),
    };

    // Check if user already liked this wish
    match check_user_liked_wish(&state.db_pool, wish_id, user_pubkey).await {
        Ok(true) => return Err(StatusCode::CONFLICT), // Already liked
        Ok(false) => {}                               // Can proceed
        Err(e) => {
            eprintln!("Database error checking like status: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Use database transaction to ensure atomicity
    let mut tx = match state.db_pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            eprintln!("Failed to start transaction: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Insert wish like record
    if let Err(e) = sqlx::query(
        r#"
        INSERT INTO wish_likes (wish_id, user_pubkey)
        VALUES (?, ?)
        ON DUPLICATE KEY UPDATE
            created_at = created_at
        "#,
    )
    .bind(wish_id)
    .bind(user_pubkey)
    .execute(&mut *tx)
    .await
    {
        eprintln!("Database error inserting wish like: {:?}", e);
        if let Err(rollback_err) = tx.rollback().await {
            eprintln!("Failed to rollback transaction: {:?}", rollback_err);
        }
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    // Increment the like count
    let new_likes = match sqlx::query_scalar::<_, i32>(
        r#"
        UPDATE wishes
        SET likes = likes + 1, updated_at = NOW()
        WHERE wish_id = ?
        "#,
    )
    .bind(wish_id)
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(_)) => {
            // Get the updated likes count
            match sqlx::query_scalar::<_, i32>("SELECT likes FROM wishes WHERE wish_id = ?")
                .bind(wish_id)
                .fetch_one(&mut *tx)
                .await
            {
                Ok(likes) => likes,
                Err(e) => {
                    eprintln!("Database error getting updated likes count: {:?}", e);
                    if let Err(rollback_err) = tx.rollback().await {
                        eprintln!("Failed to rollback transaction: {:?}", rollback_err);
                    }
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        }
        Ok(None) => {
            eprintln!("Wish not found: {}", wish_id);
            if let Err(rollback_err) = tx.rollback().await {
                eprintln!("Failed to rollback transaction: {:?}", rollback_err);
            }
            return Err(StatusCode::NOT_FOUND);
        }
        Err(e) => {
            eprintln!("Database error updating likes count: {:?}", e);
            if let Err(rollback_err) = tx.rollback().await {
                eprintln!("Failed to rollback transaction: {:?}", rollback_err);
            }
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Commit transaction
    if let Err(e) = tx.commit().await {
        eprintln!("Failed to commit transaction: {:?}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Ok(Json(json!({
        "wish_id": wish_id,
        "likes": new_likes,
        "success": true
    })))
}
