use borsh::{BorshDeserialize, BorshSerialize};
use serde::Serialize;
use solana_sdk::pubkey::Pubkey;

use crate::domain::{ActivityEnum, MedalLevel};

#[derive(Debug, BorshDeserialize)]
pub struct IncenseBought {
    pub buyer: Pubkey,
    pub incense_type: u8,
    pub number: u64,
    pub unit_price: u64,
    pub total_amount: u64,
    pub timestamp: i64,
}

#[derive(Debug, BorshDeserialize)]
pub struct IncenseBurned {
    pub user: Pubkey,
    pub incense_type: u8,
    pub nft_mint: Pubkey,
    pub incense_value: u64,
    pub merit_value: u64,
    pub timestamp: i64,
}

#[derive(Debug, BorshDeserialize)]
pub struct Donate {
    pub user: Pubkey,
    pub amount: u64,
    pub merit_value: u64,
    pub incense_value: u64,
    pub timestamp: i64,
}

#[derive(Debug, BorshDeserialize)]
pub struct MedalMinted {
    pub user: Pubkey,
    pub level: String,
    pub nft_mint: Pubkey,
    pub timestamp: i64,
}

#[derive(Debug, BorshDeserialize)]
pub struct MedalUpgraded {
    pub user: Pubkey,
    pub old_level: String,
    pub new_level: String,
    pub nft_mint: Pubkey,
    pub timestamp: i64,
}

#[derive(Debug, BorshDeserialize)]
pub struct DrawLots {
    pub user: Pubkey,
    pub lottery_type: u8,
    pub lottery_poetry: String,
    pub merit_change: u64,
    pub timestamp: i64,
}

#[derive(Debug, BorshDeserialize)]
pub struct CoinFlip {
    pub player: Pubkey,
    pub randomness_account: Pubkey,
    pub commit_slot: u64,
    pub timestamp: i64,
}

#[derive(Debug, BorshDeserialize)]
pub struct Destroy {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub timestamp: i64,
}

#[derive(Debug, BorshDeserialize)]
pub struct DonateCountCreated {
    pub authority: Pubkey,
    pub timestamp: i64,
}
#[derive(Debug, BorshDeserialize)]
pub struct TempleWithdrawal {
    pub admin: Pubkey,
    pub amount: u64,
    pub remaining_balance: u64,
    pub timestamp: i64,
}

#[derive(Debug, BorshDeserialize)]
pub struct LikeCreated {
    pub user: Pubkey,
    pub wish: Pubkey,
    pub new_like_count: u64,
    pub timestamp: i64,
}

#[derive(Debug, BorshDeserialize)]
pub struct WishCreated {
    pub user: Pubkey,
    pub content: String,
    pub value: u8,
    pub is_anonymous: bool,
    pub timestamp: i64,
}

#[derive(Debug, BorshDeserialize)]
pub struct SbtMinted {
    pub user: Pubkey,
    pub mint: Pubkey,
    pub ata: Pubkey,
    pub name: String,
    pub symbol: String,
    pub url: String,
    pub donate_amount: u64,
    pub timestamp: i64,
}

#[derive(Debug, BorshDeserialize)]
pub struct UserActivity {
    pub user: Pubkey,
    pub activity_type: ActivityEnum,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Serialize)]
pub struct UserInfo {
    pub user: Pubkey,
    pub burn_count: [u32; 6],
    pub total_burn_count: u64,
    pub incense_buy_count: [u32; 6],
    pub incense_donate_count: [u32; 6],
    pub merit_value: u64,
    pub incense_value: u64,
    pub incense_time: i64,
    pub donate_amount: u64,
    pub donate_count: u64,
    pub donate_merit_value: u64,
    pub donate_incense_value: u64,
    pub current_medal_level: MedalLevel,
    pub lottery_count: u32,
    pub lottery_is_free: bool,
    pub lottery_time: i64,
    pub wish_count: u32,
    pub wish_update_time: i64,
    pub wish_daily_count: u32,
    pub amulet_count: u64,
    pub has_sbt_token: bool,
    pub has_burn_token: [bool; 6],
}
#[derive(Debug, BorshDeserialize, BorshSerialize, Serialize)]
pub struct Temple {
    pub admin: Pubkey,
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
}
