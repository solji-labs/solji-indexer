use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShopItemType {
    Incense = 0,
    Prop = 1,
    Special = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncenseItemConfig {
    pub merit: u64,
    pub incense_points: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopItem {
    pub id: u8,
    pub name: String,
    pub description: String,
    pub price: u64,
    pub item_type: ShopItemType,
    pub stock: u64,
    pub is_available: bool,
    pub incense_config: Option<IncenseItemConfig>,
}

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
    ShopConfigUpdated {
        shop_config: Pubkey,
        temple_config: Pubkey,
        owner: Pubkey,
        shop_items: Vec<ShopItem>,
        timestamp: i64,
    },
}
