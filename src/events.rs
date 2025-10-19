use borsh::{BorshDeserialize, BorshSerialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct DonationCompleted {
    pub user: [u8; 32],
    pub amount: u64,
    pub total_donated: u64,
    pub level: u8,
    pub timestamp: i64,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct RewardsProcessed {
    pub user: [u8; 32],
    pub merit_reward: u64,
    pub incense_points_reward: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct DonationNFTMinted {
    pub user: [u8; 32],
    pub nft_mint: [u8; 32],
    pub level: u8,
    pub serial_number: u32,
    pub timestamp: i64,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct FortuneDrawn {
    pub user: [u8; 32],
    pub fortune_result: String,
    pub used_merit: bool,
    pub amulet_dropped: bool,
    pub timestamp: i64,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct FortuneNFTMinted {
    pub user: [u8; 32],
    pub fortune_nft_mint: [u8; 32],
    pub fortune_result: String,
    pub merit_cost: u32,
    pub serial_number: u32,
    pub timestamp: i64,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct WishCreated {
    pub user: [u8; 32],
    pub wish_id: u64,
    pub content_hash: [u8; 32],
    pub is_anonymous: bool,
    pub amulet_dropped: bool,
    pub timestamp: i64,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct AmuletDropped {
    pub user: [u8; 32],
    pub amulet_type: u8,
    pub source: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct AmuletMinted {
    pub user: [u8; 32],
    pub amulet_mint: [u8; 32],
    pub source: String,
    pub serial_number: u32,
    pub timestamp: i64,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct IncenseBurned {
    pub user: [u8; 32],
    pub incense_id: u8,
    pub amount: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct WishTowerUpdated {
    pub user: [u8; 32],
    pub wish_count: u32,
    pub level: u8,
    pub timestamp: i64,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ShopConfigUpdated {
    pub shop_config: [u8; 32],
    pub temple_config: [u8; 32],
    pub owner: [u8; 32],
    pub shop_items: Vec<ShopItem>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub enum ShopItemType {
    Incense = 0,
    Amulet = 1,
    Prop = 2,
    Special = 3,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct IncenseItemConfig {
    pub merit: u64,
    pub incense_points: u64,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
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

// 事件 Discriminators
pub mod discriminators {
    pub const DONATION_COMPLETED: [u8; 8] = [34, 178, 117, 6, 39, 189, 241, 48];
    pub const REWARDS_PROCESSED: [u8; 8] = [217, 74, 206, 32, 228, 181, 17, 146];
    pub const DONATION_NFT_MINTED: [u8; 8] = [142, 88, 211, 148, 62, 90, 172, 20];
    pub const FORTUNE_DRAWN: [u8; 8] = [134, 252, 88, 211, 24, 112, 209, 240];
    pub const FORTUNE_NFT_MINTED: [u8; 8] = [226, 138, 253, 243, 89, 224, 0, 199];
    pub const WISH_CREATED: [u8; 8] = [225, 167, 37, 207, 75, 1, 226, 130];
    pub const AMULET_DROPPED: [u8; 8] = [24, 100, 210, 40, 5, 63, 105, 27];
    pub const AMULET_MINTED: [u8; 8] = [5, 74, 5, 29, 227, 131, 7, 204];
    pub const INCENSE_BURNED: [u8; 8] = [211, 166, 224, 11, 104, 105, 175, 186];
    pub const WISH_TOWER_UPDATED: [u8; 8] = [32, 74, 144, 164, 26, 236, 220, 133];
    pub const SHOP_CONFIG_UPDATED: [u8; 8] = [225, 68, 254, 156, 42, 153, 151, 172];
}

// ProgramEvent

#[derive(Debug, Clone)]
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
        content_hash: [u8; 32],
        is_anonymous: bool,
        amulet_dropped: bool,
        timestamp: i64,
    },
    WishTowerUpdated {
        user: Pubkey,
        wish_count: u32,
        level: u8,
        timestamp: i64,
    },
    AmuletDropped {
        user: Pubkey,
        amulet_type: u8,
        source: String,
        timestamp: i64,
    },
    AmuletMinted {
        user: Pubkey,
        amulet_mint: Pubkey,
        source: String,
        serial_number: u32,
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
    FortuneNFTMinted {
        user: Pubkey,
        fortune_nft_mint: Pubkey,
        fortune_result: String,
        merit_cost: u32,
        serial_number: u32,
        timestamp: i64,
    },
}
