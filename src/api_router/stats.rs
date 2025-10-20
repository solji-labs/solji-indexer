// api/stats.rs

use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde_json::json;

use super::AppState;
use crate::db::get_aggregated_global_stats;

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/api/stats", get(get_global_stats))
        .with_state(state)
}

/// Get global statistics
///
/// Returns aggregated statistics about the entire temple system including
/// total merit, incense points, donations, users, wishes, and fortunes
#[utoipa::path(
    get,
    path = "/api/stats",
    responses(
        (status = 200, description = "Global statistics", body = serde_json::Value,
            example = json!({
                "total_merit": 1000000,
                "total_incense_points": 500000,
                "total_donations_sol": 1000.5,
                "total_users": 5000,
                "total_wishes": 10000,
                "total_donations": 2000,
                "total_donation_amount": 1000.5,
                "total_merit_distributed": 800000,
                "total_incense_points_distributed": 400000,
                "total_draw_fortune": 15000,
                "updated_at": 1640000000,
                "created_at": "2025-01-01T00:00:00Z"
            })
        ),
        (status = 500, description = "Internal server error")
    ),
    tag = "Statistics"
)]
pub async fn get_global_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
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
