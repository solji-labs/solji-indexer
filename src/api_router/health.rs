// api/health.rs

use axum::{response::Json, routing::get, Router};
use serde_json::json;

use super::AppState;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .with_state(state)
}

/// Health check endpoint
///
/// Returns the service status and version information
#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = serde_json::Value,
            example = json!({
                "status": "ok",
                "service": "solji-indexer",
                "version": "0.1.0"
            })
        )
    ),
    tag = "Health"
)]
pub async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "solji-indexer",
        "version": "0.1.0"
    }))
}
