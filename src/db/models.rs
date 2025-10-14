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
    pub total_donations: i32,
    pub total_donation_amount: i64,
    pub total_merit_distributed: i64,
    pub total_incense_points_distributed: i64,
    pub total_draw_fortune: i32,
    pub total_fortune_nfts: i32,
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

// UserDonationState
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserDonation {
    pub id: i32,
    pub user_pubkey: String,
    pub total_donated: f64,
    pub donation_count: i32,
    pub last_donation_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// DonationHistory
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DonationHistory {
    pub id: i32,
    pub user_pubkey: String,
    pub amount: f64, // in SOL (converted from lamports)
    pub tier: String,
    pub merit_gained: i32,
    pub transaction_signature: String,
    pub created_at: DateTime<Utc>,
}

// UserDonationBadge
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserDonationBadge {
    pub id: i32,
    pub user_pubkey: String,
    pub tier: String,
    pub badge_name: String,
    pub earned_at: DateTime<Utc>,
    pub total_donated: f64,
    pub nft_mint: Option<String>,
    pub created_at: DateTime<Utc>,
}

// IncenseLeaderboard (global leaderboard by period)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct IncenseLeaderboard {
    pub id: i32,
    pub period_type: String, // 'all', 'daily', 'weekly', 'monthly'
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

// DailyIncenseBurnCount
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DailyIncenseBurnCount {
    pub id: i32,
    pub user_pubkey: String,
    pub incense_type: i32,
    pub burn_count: i32,
    pub date: chrono::NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ShopItem
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ShopItem {
    pub id: i32,
    pub shop_config_id: String,
    pub item_id: i32,
    pub name: String,
    pub description: String,
    pub price: i64,
    pub item_type: i32,
    pub stock: i64,
    pub is_available: bool,
    pub merit: Option<i64>,
    pub incense_points: Option<i64>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// ShopConfig
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ShopConfig {
    pub id: i32,
    pub shop_config_pubkey: String,
    pub temple_config_pubkey: String,
    pub owner_pubkey: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// AmuletDropHistory
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AmuletDropHistory {
    pub id: i32,
    pub user_pubkey: String,
    pub amulet_type: i32,
    pub source: String,
    pub created_at: DateTime<Utc>,
}

// AmuletMintHistory
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AmuletMintHistory {
    pub id: i32,
    pub user_pubkey: String,
    pub amulet_mint: String,
    pub source: String,
    pub serial_number: i32,
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
