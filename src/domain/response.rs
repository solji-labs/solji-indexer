use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{types::Json, FromRow};

fn ndt_opt_as_str<S>(v: &Option<NaiveDateTime>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match v {
        Some(dt) => s.serialize_str(&dt.format("%Y-%m-%d %H:%M:%S").to_string()),
        None => s.serialize_none(),
    }
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TempleResp {
    pub admin: String, // base58
    pub level: u8,
    pub total_incense_value: u64,
    pub total_merit_value: u64,
    pub total_burn_count: u64,
    pub total_lottery_count: u64,
    pub total_wish_count: u64,
    pub total_donate_amount: u64,
    pub total_donate_count: u64,
    pub total_amulet_count: u64,
    pub buddha_nft_count: u64,
    pub wealth: u64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub create_time: Option<NaiveDateTime>,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub update_time: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct IncenseRuleResp {
    pub name: String,
    pub incense_price: u64,
    pub merit_value: u64,
    pub incense_value: u64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub create_time: Option<NaiveDateTime>,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub update_time: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoResp {
    pub pubkey: String,
    pub level: u8,
    pub burn_count: Option<Json<Vec<u32>>>,
    pub total_burn_count: u64,
    pub incense_buy_count: Option<Json<Vec<u32>>>,
    pub incense_donate_count: Option<Json<Vec<u32>>>,
    pub incense_value: u64,
    pub merit_value: u64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub incense_time: Option<NaiveDateTime>,
    pub donate_amount: u64,
    pub donate_count: u64,
    pub donate_merit_value: u64,
    pub donate_incense_value: u64,
    pub current_medal_level: u8,
    pub lottery_count: u32,
    pub lottery_is_free: bool,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub lottery_time: Option<NaiveDateTime>,
    pub wish_count: u32,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub wish_update_time: Option<NaiveDateTime>,
    pub wish_daily_count: u32,
    pub amulet_count: u64,
    pub has_sbt_token: bool,
    pub has_burn_token: Option<Json<Vec<bool>>>,
    pub stake_count: u64,
    pub tower_level: i8,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub create_time: Option<NaiveDateTime>,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub update_time: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserMedalResp {
    pub pubkey: String,
    pub current_medal_level: u8,
    pub total_merit: u64,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserDonateResp {
    pub pubkey: String,
    pub current_medal_level: u8,
    pub donate_amount: u64,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct IncenseBoughtResp {
    pub user: String,
    pub incense_type: u8,
    pub number: u64,
    pub unit_price: u64,
    pub total_amount: u64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub event_time: Option<NaiveDateTime>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct IncenseBurnedResp {
    pub user: String,
    pub incense_type: u8,
    pub nft_mint: String,
    pub incense_value: u64,
    pub merit_value: u64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub event_time: Option<NaiveDateTime>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DonateResp {
    pub user: String,
    pub amount: u64,
    pub merit_value: u64,
    pub incense_value: u64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub event_time: Option<NaiveDateTime>,
    pub timestamp: i64,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MedalMintedResp {
    pub user: String,
    pub level: String,
    pub nft_mint: String,
    pub timestamp: i64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub event_time: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct MedalUpgradedResp {
    pub user: String,
    pub old_level: String,
    pub new_level: String,
    pub nft_mint: String,
    pub timestamp: i64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub event_time: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DrawLotsResp {
    pub user: String,
    pub lottery_type: u8,
    pub lottery_poetry: String,
    pub merit_change: u64,
    pub timestamp: i64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub event_time: Option<NaiveDateTime>,
}

//===
#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CoinFlipResp {
    pub player: String,
    pub randomness_account: String,
    pub commit_slot: u64,
    pub timestamp: i64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub event_time: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DestroyResp {
    pub user: String,   // 销毁发起者
    pub mint: String,   // 被销毁的 NFT mint
    pub timestamp: i64, // 销毁时间（秒）
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub event_time: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DonateCountCreatedResp {
    pub authority: String,
    pub timestamp: i64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub event_time: Option<NaiveDateTime>,
}
#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TempleWithdrawalResp {
    pub admin: String,
    pub amount: u64,
    pub remaining_balance: u64,
    pub timestamp: i64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub event_time: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LikeCreatedResp {
    pub user: String,
    pub wish: String,
    pub new_like_count: u64,
    pub timestamp: i64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub event_time: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct WishCreatedResp {
    pub user: String,
    pub content: String,
    pub value: u8,
    pub is_anonymous: bool,
    pub timestamp: i64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub event_time: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SbtMintedResp {
    pub user: String,
    pub mint: String,
    pub ata: String,
    pub name: String,
    pub symbol: String,
    pub url: String,
    pub donate_amount: u64,
    pub timestamp: i64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub event_time: Option<NaiveDateTime>,
}
#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct UserActivityResp {
    pub user: String,
    pub activity_type: String,
    pub content: String,
    pub timestamp: i64,
    #[serde(serialize_with = "ndt_opt_as_str")]
    pub event_time: Option<NaiveDateTime>,
}
