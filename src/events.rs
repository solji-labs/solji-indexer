use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

// ProgramEvent enum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgramEvent {
    DonationCompleted {
        user: Pubkey,
        amount: u64,
        total_donated: u64,
        level: u8,
        timestamp: i64,
    },
    RewardsProcessed {
        user: Pubkey,
        merit_reward: u64,
        incense_points_reward: u64,
        timestamp: i64,
    },
    DonationNFTMinted {
        user: Pubkey,
        nft_mint: Pubkey,
        level: u8,
        serial_number: u32,
        timestamp: i64,
    },
    FortuneDrawn {
        user: Pubkey,
        fortune_result: String,
        used_merit: bool,
        amulet_dropped: bool,
        timestamp: i64,
    },
    WishCreated {
        user: Pubkey,
        wish_id: u64,
        is_anonymous: bool,
        amulet_dropped: bool,
        timestamp: i64,
    },
    IncenseBurned {
        user: Pubkey,
        incense_id: u8,
        amount: u64,
        timestamp: i64,
    },
}
