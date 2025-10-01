use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// GlobalStats
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct GlobalStats {
    pub id: i32,
    pub total_merit: i64,
    pub total_incense_points: i64,
    pub total_donations_sol: f64,
    pub total_users: i32,
    pub total_wishes: i32,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// TempleConfig
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TempleConfig {
    pub id: i32,
    pub program_id: String,
    pub incense_types: String,
    pub fortune_texts: String,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// DonationLeaderboard
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DonationLeaderboard {
    pub id: i32,
    pub program_id: String,
    pub top_donors: String,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// UserDonationState
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserDonation {
    pub id: i32,
    pub user_pubkey: String,
    pub total_donated: f64,
    pub donation_count: i32,
    pub last_donation_at: Option<i64>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// IncenseLeaderboard
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct IncenseLeaderboard {
    pub id: i32,
    pub incense_type: i32,
    pub top_users: String,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// Wish
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Wish {
    pub id: i32,
    pub wish_id: i64,
    pub user_pubkey: String,
    pub content: String,
    pub likes: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// UserState
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserState {
    pub id: i32,
    pub user_pubkey: String,
    pub has_buddha_nft: bool,
    pub has_medal_nft: bool,
    pub pending_random_request_id: Option<String>,
    pub pending_amulets: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// IncenseBurnHistory
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct IncenseBurnHistory {
    pub id: i32,
    pub user_pubkey: String,
    pub incense_type: i32,
    pub incense_amount: i32,
    pub merit_gained: i64,
    pub incense_points_gained: i64,
    pub transaction_signature: String,
    pub created_at: DateTime<Utc>,
}

// FortuneDrawHistory
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct FortuneDrawHistory {
    pub id: i32,
    pub user_pubkey: String,
    pub fortune_text: String,
    pub merit_cost: i64,
    pub is_free: bool,
    pub transaction_signature: String,
    pub created_at: DateTime<Utc>,
}

// API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalStatsResponse {
    pub total_merit: i64,
    pub total_incense_points: i64,
    pub total_donations_sol: f64,
    pub total_users: i32,
    pub total_wishes: i32,
    pub updated_at: DateTime<Utc>,
}

impl From<GlobalStats> for GlobalStatsResponse {
    fn from(stats: GlobalStats) -> Self {
        GlobalStatsResponse {
            total_merit: stats.total_merit,
            total_incense_points: stats.total_incense_points,
            total_donations_sol: stats.total_donations_sol.to_string().parse().unwrap_or(0.0),
            total_users: stats.total_users,
            total_wishes: stats.total_wishes,
            updated_at: stats.updated_at,
        }
    }
}
