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

    // TODO: Implement database operations for NFT minted
    // This would typically involve:
    // 1. Insert NFT record
    // 2. Update user state

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

    // TODO: Implement database operations for fortune drawn
    // This would typically involve:
    // 1. Insert fortune draw history
    // 2. Update user state
    // 3. Update global stats

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

    // TODO: Implement database operations for wish created
    // This would typically involve:
    // 1. Insert wish record
    // 2. Update user state
    // 3. Update global stats

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

    // TODO: Implement database operations for incense burned
    // This would typically involve:
    // 1. Insert incense burn history
    // 2. Update user state
    // 3. Update global stats
    // 4. Update leaderboard

    Ok(())
}
