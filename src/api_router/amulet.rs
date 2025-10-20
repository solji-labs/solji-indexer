// src/api/amulet.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::json;

use super::AppState;
use crate::db::{get_user_owned_amulets, get_user_pending_amulets};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route(
            "/api/amulet/user/{user_pubkey}/pending",
            get(get_user_pending),
        )
        .route(
            "/api/amulet/user/{user_pubkey}/recent-drop",
            get(get_user_recent_drop),
        )
        .route("/api/amulet/user/{user_pubkey}/owned", get(get_user_owned))
        .with_state(state)
}

/// Get user pending amulets
#[utoipa::path(
    get,
    path = "/api/amulet/user/{user_pubkey}/pending",
    params(
        ("user_pubkey" = String, Path, description = "User public key"),
    ),
    responses(
        (status = 200, description = "Pending amulets", body = serde_json::Value)
    ),
    tag = "Amulet"
)]
pub async fn get_user_pending(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match get_user_pending_amulets(&state.db_pool, &user_pubkey).await {
        Ok(amulets) => {
            let data: Vec<_> = amulets
                .into_iter()
                .map(|a| {
                    json!({
                        "id": a.id,
                        "user_pubkey": a.user_pubkey,
                        "amulet_type": a.amulet_type,
                        "source": a.source,
                        "created_at": a.created_at.to_rfc3339()
                    })
                })
                .collect();

            Ok(Json(json!({
                "user_pubkey": user_pubkey,
                "pending_amulets": data,
                "count": data.len()
            })))
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user recent amulet drop
#[utoipa::path(
    get,
    path = "/api/amulet/user/{user_pubkey}/recent-drop",
    params(
        ("user_pubkey" = String, Path, description = "User public key"),
    ),
    responses(
        (status = 200, description = "Recent amulet drop", body = serde_json::Value)
    ),
    tag = "Amulet"
)]
pub async fn get_user_recent_drop(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match get_user_pending_amulets(&state.db_pool, &user_pubkey).await {
        Ok(amulets) => {
            let recent = amulets.into_iter().next();

            let response = if let Some(a) = recent {
                json!({
                    "user_pubkey": user_pubkey,
                    "has_recent_drop": true,
                    "recent_drop": {
                        "id": a.id,
                        "amulet_type": a.amulet_type,
                        "source": a.source,
                        "created_at": a.created_at.to_rfc3339(),
                        "time_since_drop_seconds": (chrono::Utc::now() - a.created_at).num_seconds()
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
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get user owned amulets
#[utoipa::path(
    get,
    path = "/api/amulet/user/{user_pubkey}/owned",
    params(
        ("user_pubkey" = String, Path, description = "User public key"),
    ),
    responses(
        (status = 200, description = "Owned amulets", body = serde_json::Value)
    ),
    tag = "Amulet"
)]
pub async fn get_user_owned(
    State(state): State<AppState>,
    Path(user_pubkey): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match get_user_owned_amulets(&state.db_pool, &user_pubkey).await {
        Ok(amulets) => {
            let data: Vec<_> = amulets
                .into_iter()
                .map(|a| {
                    json!({
                        "id": a.id,
                        "user_pubkey": a.user_pubkey,
                        "amulet_mint": a.amulet_mint,
                        "source": a.source,
                        "serial_number": a.serial_number,
                        "created_at": a.created_at.to_rfc3339()
                    })
                })
                .collect();

            Ok(Json(json!({
                "user_pubkey": user_pubkey,
                "owned_amulets": data,
                "count": data.len()
            })))
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
