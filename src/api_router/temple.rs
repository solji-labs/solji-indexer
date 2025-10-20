// api/temple.rs

use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde_json::json;

use super::AppState;
use crate::db::{get_aggregated_global_stats, models::GlobalStats};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/api/temple/level", get(get_temple_level))
        .route("/api/temple/stats", get(get_temple_stats))
        .with_state(state)
}

/// Get temple level
///
/// Returns the current temple level based on global statistics and progress towards next level
#[utoipa::path(
    get,
    path = "/api/temple/level",
    responses(
        (status = 200, description = "Temple level information", body = serde_json::Value,
            example = json!({
                "current_level": 2,
                "level_name": "赤庙",
                "level_name_en": "Vibrant Shrine",
                "stats": {
                    "total_incense_points": 50000,
                    "total_draw_fortune": 8000,
                    "total_wishes": 5000,
                    "total_donations_sol": 200.5,
                    "total_fortune_nfts": 0
                },
                "next_level_requirements": {
                    "level": 3,
                    "level_name": "灵殿",
                    "level_name_en": "Temple of Spirit",
                    "requirements": {
                        "incense_points": 500000,
                        "draw_fortune": 30000,
                        "wishes": 10000,
                        "donations_sol": 1000.0,
                        "fortune_nfts": 0
                    }
                },
                "progress_percentage": 45.5,
                "updated_at": 1640000000
            })
        ),
        (status = 500, description = "Database error")
    ),
    tag = "Temple"
)]
pub async fn get_temple_level(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match get_aggregated_global_stats(&state.db_pool).await {
        Ok(stats) => {
            let level = calculate_temple_level(&stats);
            let next_level_requirements = get_next_level_requirements(level);

            let response = json!({
                "current_level": level,
                "level_name": get_temple_level_name(level),
                "level_name_en": get_temple_level_name_en(level),
                "stats": {
                    "total_incense_points": stats.total_incense_points,
                    "total_draw_fortune": stats.total_draw_fortune,
                    "total_wishes": stats.total_wishes,
                    "total_donations_sol": stats.total_donations_sol,
                    "total_fortune_nfts": stats.total_fortune_nfts
                },
                "next_level_requirements": next_level_requirements,
                "progress_percentage": calculate_level_progress(&stats, level),
                "updated_at": stats.updated_at.timestamp()
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get temple statistics
///
/// Returns comprehensive temple statistics including level, believers, and activities
#[utoipa::path(
    get,
    path = "/api/temple/stats",
    responses(
        (status = 200, description = "Temple statistics", body = serde_json::Value,
            example = json!({
                "level": 2,
                "level_name": "赤庙",
                "level_name_en": "Vibrant Shrine",
                "total_incense_value": 50000,
                "total_donations": 200.5,
                "total_believers": 1000,
                "total_fortunes": 8000,
                "total_wishes": 5000,
                "total_interactions": 13000,
                "updated_at": 1640000000
            })
        ),
        (status = 500, description = "Database error")
    ),
    tag = "Temple"
)]
pub async fn get_temple_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match get_aggregated_global_stats(&state.db_pool).await {
        Ok(stats) => {
            let level = calculate_temple_level(&stats);

            let response = json!({
                "level": level,
                "level_name": get_temple_level_name(level),
                "level_name_en": get_temple_level_name_en(level),
                "total_incense_value": stats.total_incense_points,
                "total_donations": stats.total_donations_sol,
                "total_believers": stats.total_users,
                "total_fortunes": stats.total_draw_fortune,
                "total_wishes": stats.total_wishes,
                "total_interactions": stats.total_draw_fortune + stats.total_wishes,
                "updated_at": stats.updated_at.timestamp()
            });
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Database error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Helper functions

fn calculate_temple_level(stats: &GlobalStats) -> u8 {
    let requirements = vec![
        (1, 0, 0, 0, 0.0, 0),
        (2, 10000, 5000, 3000, 100.0, 0),
        (3, 500000, 30000, 10000, 1000.0, 0),
        (4, 1000000, 100000, 50000, 5000.0, 0),
    ];

    for (level, req_incense, req_fortune, req_wishes, req_donations, req_nfts) in
        requirements.into_iter().rev()
    {
        if stats.total_incense_points >= req_incense as i64
            && stats.total_draw_fortune >= req_fortune as i32
            && stats.total_wishes >= req_wishes as i32
            && stats.total_donations_sol >= req_donations
            && stats.total_fortune_nfts >= req_nfts as i32
        {
            return level;
        }
    }
    1
}

fn get_next_level_requirements(current_level: u8) -> serde_json::Value {
    match current_level {
        1 => json!({
            "level": 2,
            "level_name": "赤庙",
            "level_name_en": "Vibrant Shrine",
            "requirements": {
                "incense_points": 10000,
                "draw_fortune": 5000,
                "wishes": 3000,
                "donations_sol": 100.0,
                "fortune_nfts": 0
            }
        }),
        2 => json!({
            "level": 3,
            "level_name": "灵殿",
            "level_name_en": "Temple of Spirit",
            "requirements": {
                "incense_points": 500000,
                "draw_fortune": 30000,
                "wishes": 10000,
                "donations_sol": 1000.0,
                "fortune_nfts": 0
            }
        }),
        3 => json!({
            "level": 4,
            "level_name": "赛博神殿",
            "level_name_en": "Cyber Shrine",
            "requirements": {
                "incense_points": 1000000,
                "draw_fortune": 100000,
                "wishes": 50000,
                "donations_sol": 5000.0,
                "fortune_nfts": 0
            }
        }),
        _ => json!({
            "level": null,
            "message": "Maximum level reached"
        }),
    }
}

fn calculate_level_progress(stats: &GlobalStats, current_level: u8) -> f64 {
    if current_level >= 4 {
        return 100.0;
    }

    let next_requirements = match current_level {
        1 => (10000, 5000, 3000, 100.0),
        2 => (500000, 30000, 10000, 1000.0),
        3 => (1000000, 100000, 50000, 5000.0),
        _ => return 100.0,
    };

    let (req_incense, req_fortune, req_wishes, req_donations) = next_requirements;

    let incense_progress = (stats.total_incense_points as f64 / req_incense as f64).min(1.0);
    let fortune_progress = (stats.total_draw_fortune as f64 / req_fortune as f64).min(1.0);
    let wishes_progress = (stats.total_wishes as f64 / req_wishes as f64).min(1.0);
    let donations_progress = (stats.total_donations_sol / req_donations).min(1.0);

    (incense_progress + fortune_progress + wishes_progress + donations_progress) / 4.0 * 100.0
}

fn get_temple_level_name(level: u8) -> &'static str {
    match level {
        1 => "草庙",
        2 => "赤庙",
        3 => "灵殿",
        4 => "赛博神殿",
        _ => "未知",
    }
}

fn get_temple_level_name_en(level: u8) -> &'static str {
    match level {
        1 => "Rustic Shrine",
        2 => "Vibrant Shrine",
        3 => "Temple of Spirit",
        4 => "Cyber Shrine",
        _ => "Unknown",
    }
}
