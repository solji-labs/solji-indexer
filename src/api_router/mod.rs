// api/mod.rs
use axum::Router;
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::OpenApi;

use crate::db::DbPool;
use crate::indexer::fetcher::IndexerFetcher;
use crate::utils::config::Config;

// 子模块
pub mod amulet;
pub mod donation;
pub mod fortune;
pub mod health;
pub mod incense;
pub mod ipfs;
pub mod models;
pub mod profile;
pub mod stats;
pub mod temple;
pub mod wishes;

#[derive(Clone)]
pub struct AppState {
    pub fetcher: Arc<RwLock<IndexerFetcher>>,
    pub config: Config,
    pub db_pool: DbPool,
}

/// 创建主路由
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health
        .merge(health::routes(state.clone()))
        // Stats
        .merge(stats::routes(state.clone()))
        // Temple
        .merge(temple::routes(state.clone()))
        // Incense
        .merge(incense::routes(state.clone()))
        // Wishes
        .merge(wishes::routes(state.clone()))
        // Donation
        .merge(donation::routes(state.clone()))
        // Amulet
        .merge(amulet::routes(state.clone()))
        // Fortune
        .merge(fortune::routes(state.clone()))
        // Profile
        .merge(profile::routes(state.clone()))
        // IPFS
        .merge(ipfs::routes(state.clone()))
}

/// OpenAPI 文档汇总
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Solji Indexer API",
        description = "Solji Temple Indexer API for tracking blockchain events and providing temple statistics",
        version = "0.1.0",
        contact(
            name = "Solji Team",
            url = "https://solji.temple"
        )
    ),
    paths(
        // Health
        health::health_check,

        // Stats
        stats::get_global_stats,

        // Temple
        temple::get_temple_level,
        temple::get_temple_stats,
        temple::get_recent_activities,

        // Incense
        incense::get_incense_types,
        incense::check_can_burn_incense,
        incense::get_user_burn_count,
        incense::get_user_nfts,
        incense::get_user_history,
        incense::get_leaderboard,

        // Wishes
        wishes::get_wishes,
        wishes::get_public_wishes,
        wishes::get_user_wishes,
        wishes::get_user_daily_count,
        wishes::get_user_tower,
        wishes::like_wish,

        // Donation
        donation::get_tiers,
        donation::get_leaderboard,
        donation::check_top_donors,
        donation::get_user_history,
        donation::get_user_badges,
        donation::get_honor_wall,
        donation::submit_transaction,

        // Amulet
        amulet::get_user_pending,
        amulet::get_user_recent_drop,
        amulet::get_user_owned,

        // Fortune
        fortune::get_fortune_leaderboard_handler,
        fortune::get_user_fortune_history_handler,
        fortune::get_user_fortune_nft_mints_handler,
        fortune::get_user_fortune_stats_handler,

        // Profile
        profile::get_profile_basic,
        profile::get_profile_activities,
        profile::get_profile_achievements,
        profile::get_profile_nfts,

        // IPFS
        ipfs::upload_to_ipfs,
        ipfs::get_ipfs_content,
        ipfs::get_ipfs_batch_content,
    ),
    components(
        schemas(
            models::IncenseTypeInfo,
            models::DonationTierInfo,
            models::DonationTransactionRequest,
            models::PaginationParams,
            models::ErrorResponse,
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Statistics", description = "Global statistics endpoints"),
        (name = "Temple", description = "Temple level and statistics"),
        (name = "Incense", description = "Incense burning related endpoints"),
        (name = "Wishes", description = "Wish making and management"),
        (name = "Donation", description = "Donation and merit system"),
        (name = "Amulet", description = "Amulet NFT management"),
        (name = "Fortune", description = "Fortune drawing and statistics"),
        (name = "Profile", description = "User profile and statistics"),
        (name = "IPFS", description = "IPFS content upload and management"),
        (name = "Admin", description = "Administrative endpoints"),
    )
)]
pub struct ApiDoc;
