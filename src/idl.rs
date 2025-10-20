use anchor_lang::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DonationCompleted {
    pub user: Pubkey,
    pub amount: u64,
    pub total_donated: u64,
    pub level: u8,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct DonationNFTMinted {
    pub user: Pubkey,
    pub nft_mint: Pubkey,
    pub level: u8,
    pub serial_number: u32,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FortuneDrawn {
    pub user: Pubkey,
    pub fortune_result: String,
    pub used_merit: bool,
    pub amulet_dropped: bool,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RewardsProcessed {
    pub user: Pubkey,
    pub merit_reward: u64,
    pub incense_points_reward: u64,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct WishCreated {
    pub user: Pubkey,
    pub wish_id: u64,
    pub content_hash: [u8; 32],
    pub is_anonymous: bool,
    pub amulet_dropped: bool,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AmuletDropped {
    pub user: Pubkey,
    pub amulet_type: u8,
    pub source: String,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AmuletMinted {
    pub user: Pubkey,
    pub amulet_mint: Pubkey,
    pub source: String,
    pub serial_number: u32,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct IncenseBurned {
    pub user: Pubkey,
    pub incense_id: u8,
    pub amount: u64,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FortuneNFTMinted {
    pub user: Pubkey,
    pub fortune_nft_mint: Pubkey,
    pub fortune_result: String,
    pub merit_cost: u32,
    pub serial_number: u32,
    pub timestamp: i64,
}

// 事件识别码常量
pub const DONATION_COMPLETED_DISCRIMINATOR: [u8; 8] = [34, 178, 117, 6, 39, 189, 241, 48];
pub const DONATION_NFT_MINTED_DISCRIMINATOR: [u8; 8] = [142, 88, 211, 148, 62, 90, 172, 20];
pub const FORTUNE_DRAWN_DISCRIMINATOR: [u8; 8] = [134, 252, 88, 211, 24, 112, 209, 240];
pub const FORTUNE_NFT_MINTED_DISCRIMINATOR: [u8; 8] = [226, 138, 253, 243, 89, 224, 0, 199];
pub const AMULET_DROPPED_DISCRIMINATOR: [u8; 8] = [24, 100, 210, 40, 5, 63, 105, 27];
pub const AMULET_MINTED_DISCRIMINATOR: [u8; 8] = [5, 74, 5, 29, 227, 131, 7, 204];
pub const REWARDS_PROCESSED_DISCRIMINATOR: [u8; 8] = [217, 74, 206, 32, 228, 181, 17, 146];
pub const WISH_CREATED_DISCRIMINATOR: [u8; 8] = [225, 167, 37, 207, 75, 1, 226, 130];
pub const INCENSE_BURNED_DISCRIMINATOR: [u8; 8] = [211, 166, 224, 11, 104, 105, 175, 186];
