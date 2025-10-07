use sqlx::MySql;
use sqlx::Pool;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;

use crate::db::DbPool;
use crate::events::ProgramEvent;

/// Start event processor worker
pub async fn start_event_processor(id: usize, mut receiver: Receiver<ProgramEvent>, pool: DbPool) {
    println!("Worker #{} started.", id);

    // Worker loop: continuously receive events from channel
    while let Some(event) = receiver.recv().await {
        // Core: call database handling functions for each event type
        match event {
            ProgramEvent::DonationCompleted {
                user,
                amount,
                total_donated,
                level,
                timestamp,
            } => {
                if let Err(err) =
                    handle_donation_completed(&pool, user, amount, total_donated, level, timestamp)
                        .await
                {
                    eprintln!(
                        "Worker #{}: Failed to handle DonationCompleted: {:?}",
                        id, err
                    );
                }
            }
            ProgramEvent::RewardsProcessed {
                user,
                merit_reward,
                incense_points_reward,
                timestamp,
            } => {
                if let Err(err) = handle_rewards_processed(
                    &pool,
                    user,
                    merit_reward,
                    incense_points_reward,
                    timestamp,
                )
                .await
                {
                    eprintln!(
                        "Worker #{}: Failed to handle RewardsProcessed: {:?}",
                        id, err
                    );
                }
            }
            ProgramEvent::DonationNFTMinted {
                user,
                nft_mint,
                level,
                serial_number,
                timestamp,
            } => {
                if let Err(err) = handle_donation_nft_minted(
                    &pool,
                    user,
                    nft_mint,
                    level,
                    serial_number,
                    timestamp,
                )
                .await
                {
                    eprintln!(
                        "Worker #{}: Failed to handle DonationNFTMinted: {:?}",
                        id, err
                    );
                }
            }
            ProgramEvent::FortuneDrawn {
                user,
                fortune_result,
                used_merit,
                amulet_dropped,
                timestamp,
            } => {
                if let Err(err) = handle_fortune_drawn(
                    &pool,
                    user,
                    fortune_result,
                    used_merit,
                    amulet_dropped,
                    timestamp,
                )
                .await
                {
                    eprintln!("Worker #{}: Failed to handle FortuneDrawn: {:?}", id, err);
                }
            }
            ProgramEvent::WishCreated {
                user,
                wish_id,
                is_anonymous,
                amulet_dropped,
                timestamp,
            } => {
                if let Err(err) = handle_wish_created(
                    &pool,
                    user,
                    wish_id,
                    is_anonymous,
                    amulet_dropped,
                    timestamp,
                )
                .await
                {
                    eprintln!("Worker #{}: Failed to handle WishCreated: {:?}", id, err);
                }
            }
            ProgramEvent::IncenseBurned {
                user,
                incense_id,
                amount,
                timestamp,
            } => {
                if let Err(err) =
                    handle_incense_burned(&pool, user, incense_id, amount, timestamp).await
                {
                    eprintln!("Worker #{}: Failed to handle IncenseBurned: {:?}", id, err);
                }
            }
            ProgramEvent::AmuletDropped {
                user,
                source,
                timestamp,
            } => {
                if let Err(err) = handle_amulet_dropped(&pool, user, source, timestamp).await {
                    eprintln!("Worker #{}: Failed to handle AmuletDropped: {:?}", id, err);
                }
            }
            ProgramEvent::AmuletMinted {
                user,
                amulet_mint,
                source,
                serial_number,
                timestamp,
            } => {
                if let Err(err) =
                    handle_amulet_minted(&pool, user, amulet_mint, source, serial_number, timestamp)
                        .await
                {
                    eprintln!("Worker #{}: Failed to handle AmuletMinted: {:?}", id, err);
                }
            }
            ProgramEvent::ShopConfigUpdated {
                shop_config,
                temple_config,
                owner,
                shop_items,
                timestamp,
            } => {
                if let Err(err) = handle_shop_config_updated(
                    &pool,
                    shop_config,
                    temple_config,
                    owner,
                    shop_items,
                    timestamp,
                )
                .await
                {
                    eprintln!(
                        "Worker #{}: Failed to handle ShopConfigUpdated: {:?}",
                        id, err
                    );
                }
            }
        }
    }

    println!("Worker #{} finished.", id);
}

/// Handle DonationCompleted event
async fn handle_donation_completed(
    pool: &DbPool,
    user: solana_sdk::pubkey::Pubkey,
    amount: u64,
    total_donated: u64,
    level: u8,
    timestamp: i64,
) -> Result<(), sqlx::Error> {
    println!(
        "Processing DonationCompleted: user={}, amount={}, total_donated={}, level={}",
        user, amount, total_donated, level
    );

    let user_str = user.to_string();

    // 1. Calculate rewards based on donation level
    let (merit_reward, incense_reward) = crate::rewards::calculate_donation_rewards(level);

    // 2. Atomically update user state (incremental update)
    crate::db::upsert_user_state_by_donation(pool, &user_str, merit_reward, incense_reward, amount)
        .await?;

    // 3. Atomically update global stats
    crate::db::upsert_global_stats_by_donation(pool, merit_reward, incense_reward, amount).await?;

    // 4. Insert donation history (if needed)
    // Note: You might want to add a donation history table for detailed tracking

    Ok(())
}

/// Handle RewardsProcessed event
async fn handle_rewards_processed(
    pool: &DbPool,
    user: solana_sdk::pubkey::Pubkey,
    merit_reward: u64,
    incense_points_reward: u64,
    timestamp: i64,
) -> Result<(), sqlx::Error> {
    println!(
        "Processing RewardsProcessed: user={}, merit_reward={}, incense_points_reward={}",
        user, merit_reward, incense_points_reward
    );

    let user_str = user.to_string();

    // 1. Atomically update user state with rewards
    crate::db::upsert_user_state_by_rewards(pool, &user_str, merit_reward, incense_points_reward)
        .await?;

    // 2. Atomically update global stats
    crate::db::upsert_global_stats_by_rewards(pool, merit_reward, incense_points_reward).await?;

    Ok(())
}

/// Handle DonationNFTMinted event
async fn handle_donation_nft_minted(
    pool: &DbPool,
    user: solana_sdk::pubkey::Pubkey,
    nft_mint: solana_sdk::pubkey::Pubkey,
    level: u8,
    serial_number: u32,
    timestamp: i64,
) -> Result<(), sqlx::Error> {
    println!(
        "Processing DonationNFTMinted: user={}, nft_mint={}, level={}, serial_number={}",
        user, nft_mint, level, serial_number
    );

    // NFT ownership is now checked via ATA (Associated Token Account)
    // No database updates needed for this event

    Ok(())
}

/// Handle FortuneDrawn event
async fn handle_fortune_drawn(
    pool: &DbPool,
    user: solana_sdk::pubkey::Pubkey,
    fortune_result: String,
    used_merit: bool,
    amulet_dropped: bool,
    timestamp: i64,
) -> Result<(), sqlx::Error> {
    println!(
        "Processing FortuneDrawn: user={}, fortune_result={}, used_merit={}, amulet_dropped={}",
        user, fortune_result, used_merit, amulet_dropped
    );

    let user_str = user.to_string();
    let created_at =
        chrono::DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| chrono::Utc::now());

    // 1. Insert fortune draw history
    crate::db::insert_fortune_draw_history(
        &pool,
        &user_str,
        &fortune_result,
        if used_merit { 100 } else { 0 }, // Assuming merit cost is 100
        !used_merit,
        "", // transaction_signature - would need to be passed from event
    )
    .await?;

    // 2. Update user state (increment fortune draws count)
    crate::db::increment_user_fortune_draws(&pool, &user_str, created_at).await?;

    // 3. Update global stats
    crate::db::increment_global_stats_fortune_draws(&pool, created_at).await?;

    Ok(())
}

/// Handle WishCreated event
async fn handle_wish_created(
    pool: &DbPool,
    user: solana_sdk::pubkey::Pubkey,
    wish_id: u64,
    is_anonymous: bool,
    amulet_dropped: bool,
    timestamp: i64,
) -> Result<(), sqlx::Error> {
    println!(
        "Processing WishCreated: user={}, wish_id={}, is_anonymous={}, amulet_dropped={}",
        user, wish_id, is_anonymous, amulet_dropped
    );

    let user_str = user.to_string();
    let created_at =
        chrono::DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| chrono::Utc::now());

    // 1. Insert wish record (content would need to be passed from event)
    // For now, we'll insert with placeholder content
    crate::db::upsert_wish(
        &pool,
        wish_id,
        &user_str,
        "Wish content", // TODO: Pass actual content from event
        0,              // initial likes
        created_at,
        created_at,
    )
    .await?;

    // 2. Update user state (increment wish count)
    crate::db::increment_user_wish_count(&pool, &user_str, created_at).await?;

    // 3. Update global stats
    crate::db::increment_global_stats_wishes(&pool, created_at).await?;

    Ok(())
}

/// Handle IncenseBurned event
async fn handle_incense_burned(
    pool: &DbPool,
    user: solana_sdk::pubkey::Pubkey,
    incense_id: u8,
    amount: u64,
    timestamp: i64,
) -> Result<(), sqlx::Error> {
    println!(
        "Processing IncenseBurned: user={}, incense_id={}, amount={}",
        user, incense_id, amount
    );

    let user_str = user.to_string();
    let created_at =
        chrono::DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| chrono::Utc::now());

    // // Check daily limit before processing
    // let can_burn =
    //     crate::db::check_daily_incense_limit(&pool, &user_str, incense_id as i32, amount as i32)
    //         .await?;

    // if !can_burn {
    //     println!(
    //         "Daily limit exceeded for user {} incense type {}, amount {}",
    //         user_str, incense_id, amount
    //     );

    // }

    // Calculate rewards based on incense type (this should match the contract logic)
    // For now, we'll use placeholder values - in real implementation, this should
    // query the temple config to get the exact merit and incense_points values
    let merit_gained = 10 * amount as i64; // Placeholder: 10 merit per incense
    let incense_points_gained = 100 * amount as i64; // Placeholder: 100 incense points per incense

    // Update user incense balance, insert history, and update global stats atomically
    crate::db::update_user_incense_and_history(
        &pool,
        &user_str,
        incense_id as i32,
        amount as i32,
        merit_gained,
        incense_points_gained,
        "", // transaction_signature - would need to be passed from event
        created_at,
    )
    .await?;

    Ok(())
}

/// Handle AmuletDropped event
async fn handle_amulet_dropped(
    pool: &DbPool,
    user: solana_sdk::pubkey::Pubkey,
    source: String,
    timestamp: i64,
) -> Result<(), sqlx::Error> {
    println!("Processing AmuletDropped: user={}, source={}", user, source);

    let user_str = user.to_string();
    let created_at =
        chrono::DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| chrono::Utc::now());

    // 1. Insert amulet drop history
    crate::db::insert_amulet_drop_history(&pool, &user_str, &source, created_at).await?;

    // 2. Update user amulet collection stats
    crate::db::increment_user_amulet_stats(&pool, &user_str, &source, created_at).await?;

    Ok(())
}

/// Handle AmuletMinted event
async fn handle_amulet_minted(
    pool: &DbPool,
    user: solana_sdk::pubkey::Pubkey,
    amulet_mint: solana_sdk::pubkey::Pubkey,
    source: String,
    serial_number: u32,
    timestamp: i64,
) -> Result<(), sqlx::Error> {
    println!(
        "Processing AmuletMinted: user={}, amulet_mint={}, source={}, serial_number={}",
        user, amulet_mint, source, serial_number
    );

    let user_str = user.to_string();
    let amulet_mint_str = amulet_mint.to_string();
    let created_at =
        chrono::DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| chrono::Utc::now());

    // 1. Insert amulet mint history
    crate::db::insert_amulet_mint_history(
        &pool,
        &user_str,
        &amulet_mint_str,
        &source,
        serial_number,
        created_at,
    )
    .await?;

    // 2. Update user amulet collection stats (decrement pending, increment total)
    crate::db::decrement_user_pending_amulets(&pool, &user_str, created_at).await?;

    Ok(())
}

/// Handle ShopConfigUpdated event
async fn handle_shop_config_updated(
    pool: &DbPool,
    shop_config: solana_sdk::pubkey::Pubkey,
    temple_config: solana_sdk::pubkey::Pubkey,
    owner: solana_sdk::pubkey::Pubkey,
    shop_items: Vec<crate::events::ShopItem>,
    timestamp: i64,
) -> Result<(), sqlx::Error> {
    println!(
        "Processing ShopConfigUpdated: shop_config={}, temple_config={}, owner={}, items_count={}",
        shop_config,
        temple_config,
        owner,
        shop_items.len()
    );

    let shop_config_str = shop_config.to_string();
    let temple_config_str = temple_config.to_string();
    let owner_str = owner.to_string();
    let updated_at =
        chrono::DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| chrono::Utc::now());

    // 1. Upsert shop config record
    crate::db::upsert_shop_config(
        pool,
        &shop_config_str,
        &temple_config_str,
        &owner_str,
        updated_at,
    )
    .await?;

    // 2. Sync shop items (delete existing and insert new ones)
    crate::db::sync_shop_items(pool, &shop_config_str, &shop_items, updated_at).await?;

    Ok(())
}
