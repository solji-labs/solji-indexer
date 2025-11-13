use tokio::sync::mpsc::Receiver;

use crate::db::insert_fortune_nft_mint_history;
use crate::db::DbPool;
use crate::events::ProgramEvent;

/// Get current time in CST (UTC+8)
fn get_current_cst_time() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now() + chrono::Duration::hours(8)
}

/// Extract event type and transaction signature from ProgramEvent
fn get_event_info(event: &ProgramEvent) -> (&str, String) {
    match event {
        ProgramEvent::DonationCompleted {
            transaction_signature,
            ..
        } => ("DonationCompleted", transaction_signature.clone()),
        ProgramEvent::RewardsProcessed {
            transaction_signature,
            ..
        } => ("RewardsProcessed", transaction_signature.clone()),
        ProgramEvent::DonationNFTMinted {
            transaction_signature,
            ..
        } => ("DonationNFTMinted", transaction_signature.clone()),
        ProgramEvent::FortuneDrawn {
            transaction_signature,
            ..
        } => ("FortuneDrawn", transaction_signature.clone()),
        ProgramEvent::WishCreated {
            transaction_signature,
            ..
        } => ("WishCreated", transaction_signature.clone()),
        ProgramEvent::WishTowerUpdated {
            transaction_signature,
            ..
        } => ("WishTowerUpdated", transaction_signature.clone()),
        ProgramEvent::IncenseBurned {
            transaction_signature,
            ..
        } => ("IncenseBurned", transaction_signature.clone()),
        ProgramEvent::AmuletDropped {
            transaction_signature,
            ..
        } => ("AmuletDropped", transaction_signature.clone()),
        ProgramEvent::AmuletMinted {
            transaction_signature,
            ..
        } => ("AmuletMinted", transaction_signature.clone()),
        ProgramEvent::ShopConfigUpdated {
            transaction_signature,
            ..
        } => ("ShopConfigUpdated", transaction_signature.clone()),
        ProgramEvent::FortuneNFTMinted {
            transaction_signature,
            ..
        } => ("FortuneNFTMinted", transaction_signature.clone()),
    }
}

/// Start event processor worker
pub async fn start_event_processor(id: usize, mut receiver: Receiver<ProgramEvent>, pool: DbPool) {
    println!("Worker #{} started.", id);

    // Worker loop: continuously receive events from channel
    while let Some(event) = receiver.recv().await {
        // Core: call database handling functions for each event type
        // Check for duplicate events
        let (event_type, transaction_signature) = get_event_info(&event);

        // Check if this transaction has already been processed
        match crate::db::is_transaction_processed(&pool, &transaction_signature, event_type).await {
            Ok(true) => {
                println!(
                    "Skipping duplicate event: {} for transaction {}",
                    event_type, transaction_signature
                );
                continue;
            }
            Ok(false) => {
                // Mark as processed
                if let Err(e) =
                    crate::db::mark_transaction_processed(&pool, &transaction_signature, event_type)
                        .await
                {
                    eprintln!("Failed to mark transaction as processed: {:?}", e);
                    continue;
                }
            }
            Err(e) => {
                eprintln!("Failed to check transaction processing status: {:?}", e);
                continue;
            }
        }

        match event {
            ProgramEvent::DonationCompleted {
                user,
                amount,
                total_donated,
                level,
                timestamp,
                ..
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
                ..
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
                ..
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
                ..
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
                content_hash,
                is_anonymous,
                amulet_dropped,
                timestamp,
                ..
            } => {
                if let Err(err) = handle_wish_created(
                    &pool,
                    user,
                    wish_id,
                    &content_hash,
                    is_anonymous,
                    amulet_dropped,
                    timestamp,
                )
                .await
                {
                    eprintln!("Worker #{}: Failed to handle WishCreated: {:?}", id, err);
                }
            }
            ProgramEvent::WishTowerUpdated {
                user,
                wish_count,
                level,
                timestamp,
                ..
            } => {
                if let Err(err) =
                    handle_wish_tower_updated(&pool, user, wish_count, level, timestamp).await
                {
                    eprintln!(
                        "Worker #{}: Failed to handle WishTowerUpdated: {:?}",
                        id, err
                    );
                }
            }
            ProgramEvent::IncenseBurned {
                user,
                incense_id,
                amount,
                timestamp,
                ..
            } => {
                if let Err(err) =
                    handle_incense_burned(&pool, user, incense_id, amount, timestamp).await
                {
                    eprintln!("Worker #{}: Failed to handle IncenseBurned: {:?}", id, err);
                }
            }
            ProgramEvent::AmuletDropped {
                user,
                amulet_type,
                source,
                timestamp,
                ..
            } => {
                if let Err(err) =
                    handle_amulet_dropped(&pool, user, amulet_type, source, timestamp).await
                {
                    eprintln!("Worker #{}: Failed to handle AmuletDropped: {:?}", id, err);
                }
            }
            ProgramEvent::AmuletMinted {
                user,
                amulet_mint,
                source,
                serial_number,
                timestamp,
                ..
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
                ..
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
            ProgramEvent::FortuneNFTMinted {
                user,
                fortune_nft_mint,
                fortune_result,
                merit_cost,
                serial_number,
                timestamp,
                ..
            } => {
                if let Err(err) = handle_fortune_nft_minted(
                    &pool,
                    user,
                    fortune_nft_mint,
                    fortune_result,
                    merit_cost,
                    serial_number,
                    timestamp,
                )
                .await
                {
                    eprintln!(
                        "Worker #{}: Failed to handle FortuneNFTMinted: {:?}",
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
    let amount_sol = amount as f64 / 1_000_000_000.0;

    // Map level to tier string
    let tier = match level {
        1 => "bronze",
        2 => "silver",
        3 => "gold",
        4 => "supreme",
        _ => "none",
    };

    // Calculate merit gained based on level (from program logic)
    let merit_gained = match level {
        1 => 65,
        2 => 1300,
        3 => 14000,
        4 => 120000,
        _ => 0,
    };

    // 1. Insert donation history record
    match crate::db::insert_donation_history(
        pool,
        &user_str,
        amount_sol,
        tier,
        merit_gained as i32,
        "", // transaction_signature - TODO: get from transaction data
    )
    .await
    {
        Ok(_) => println!("✅ Successfully inserted donation history record"),
        Err(e) => {
            eprintln!("❌ Failed to insert donation history record: {:?}", e);
            return Err(e);
        }
    }

    // 2. Update user donation record
    let current_time = get_current_cst_time();
    match crate::db::upsert_user_donation(
        pool,
        &user_str,
        amount,
        1, // donation_count
        Some(current_time),
        current_time,
    )
    .await
    {
        Ok(_) => println!("✅ Successfully updated user donation record"),
        Err(e) => {
            eprintln!("❌ Failed to update user donation record: {:?}", e);
            return Err(e);
        }
    }

    // 3. Update user state with donation amount
    match crate::db::upsert_user_state_by_donation(
        pool,
        &user_str,
        merit_gained as i64,
        0,      // no incense points
        amount, // donation amount in lamports
    )
    .await
    {
        Ok(_) => println!("✅ Successfully updated user state with donation"),
        Err(e) => {
            eprintln!("❌ Failed to update user state with donation: {:?}", e);
            return Err(e);
        }
    }

    // All rewards handled in contract transaction

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

    crate::db::upsert_user_state_by_donation(
        pool,
        &user_str,
        merit_reward as i64,
        incense_points_reward,
        0,
    )
    .await?;

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
    let created_at = get_current_cst_time();

    // 1. Insert fortune draw history
    crate::db::insert_fortune_draw_history(
        &pool,
        &user_str,
        &fortune_result,
        if used_merit { 5 } else { 0 }, // Contract uses 5 merit for paid draws
        !used_merit,
        "", // transaction_signature - would need to be passed from event
    )
    .await?;

    // 2. Update user state (increment fortune draws count)
    crate::db::increment_user_fortune_draws(&pool, &user_str, created_at).await?;

    // 3. Handle merit changes - All fortune draws give +2 merit reward
    if used_merit {
        // Paid fortune draw: deduct 5 merit + reward 2 merit = net -3 merit
        crate::db::upsert_user_state_by_donation(
            pool, &user_str, -3i64, // -5 + 2 = -3 merit
            0,     // no incense points
            0,     // no donation amount
        )
        .await?;
        println!(
            "Paid fortune draw: deducted 5 merit, added 2 merit reward (net -3) for user {}",
            user_str
        );
    } else {
        // Free fortune draw: reward 2 merit
        crate::db::upsert_user_state_by_donation(
            pool, &user_str, 2i64, // +2 merit for free draw
            0,    // no incense points
            0,    // no donation amount
        )
        .await?;
        println!(
            "Free fortune draw: added 2 merit reward for user {}",
            user_str
        );
    }

    Ok(())
}

/// Handle WishCreated event
async fn handle_wish_created(
    pool: &DbPool,
    user: solana_sdk::pubkey::Pubkey,
    wish_id: u64,
    content_hash: &[u8; 32],
    is_anonymous: bool,
    amulet_dropped: bool,
    timestamp: i64,
) -> Result<(), sqlx::Error> {
    println!(
        "Processing WishCreated: user={}, wish_id={}, content_hash_len={}, is_anonymous={}, amulet_dropped={}",
        user, wish_id, content_hash.len(), is_anonymous, amulet_dropped
    );

    let user_str = user.to_string();
    let created_at = get_current_cst_time();

    // Convert content_hash to hex string for storage
    let content_hash_hex = hex::encode(content_hash);

    // 1. Check if user has already made 3 wishes today
    let daily_wish_count = crate::db::get_user_daily_wish_count(&pool, &user_str).await?;
    let is_paid_wish = daily_wish_count >= 3;

    // 2. Insert wish record with content hash
    crate::db::upsert_wish(
        &pool,
        wish_id,
        &user_str,
        &content_hash_hex, // Store content hash as hex string
        0,                 // initial likes
        created_at,
        created_at,
    )
    .await?;

    // 3. Handle merit changes
    // All wishes give +1 merit reward
    if is_paid_wish {
        crate::db::upsert_user_state_by_donation(
            pool, &user_str, -4i64, // -5 + 1 = -4
            0,     // no incense points
            0,     // no donation amount
        )
        .await?;
        println!(
            "Paid wish: deducted 5 merit, added 1 merit reward (net -4) for user {}",
            user_str
        );
    } else {
        crate::db::upsert_user_state_by_donation(
            pool, &user_str, 1i64, // +1 merit for making a wish
            0,    // no incense points
            0,    // no donation amount
        )
        .await?;
        println!("Free wish: added 1 merit reward for user {}", user_str);
    }

    // 4. Increment user wish count
    crate::db::increment_user_wish_count(&pool, &user_str, created_at).await?;

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
    let created_at = get_current_cst_time();

    // Check daily limit before processing
    let can_burn =
        crate::db::check_daily_incense_limit(&pool, &user_str, incense_id as i32, amount as i32)
            .await?;

    if !can_burn {
        println!(
            "Daily limit exceeded for user {} incense type {}, amount {}",
            user_str, incense_id, amount
        );
        // Note: Since the contract should have prevented this, this might indicate
        // a synchronization issue or contract bug. We'll still process the event
        // but log the discrepancy.
    }

    // Calculate rewards based on incense type (matching the product specifications)
    let (merit_gained, incense_points_gained) = match incense_id {
        0 => (10 * amount as i64, 100 * amount as i64), // Clear Incense: 10 merit, 100 incense points
        1 => (65 * amount as i64, 600 * amount as i64), // Sandalwood: 65 merit, 600 incense points
        2 => (1200 * amount as i64, 3100 * amount as i64), // Ambergris Incense: 1200 merit, 3100 incense points
        3 => (3400 * amount as i64, 9000 * amount as i64), // Supreme Spirit Incense: 3400 merit, 9000 incense points
        4 => (12000 * amount as i64, 10000 * amount as i64), // Secret Brew Incense: 12000 merit, 10000 incense points
        5 => (300000 * amount as i64, 400000 * amount as i64), // Celestial Incense: 300000 merit, 400000 incense points
        _ => (10 * amount as i64, 100 * amount as i64),        // Default values
    };

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

    // Update incense leaderboard for all periods
    if let Err(err) = crate::db::update_incense_leaderboard_all_periods(&pool, created_at).await {
        eprintln!("Failed to update incense leaderboard: {:?}", err);
        // Don't fail the entire event processing for leaderboard update failure
    }

    Ok(())
}

/// Handle AmuletDropped event
async fn handle_amulet_dropped(
    pool: &DbPool,
    user: solana_sdk::pubkey::Pubkey,
    amulet_type: u8,
    source: String,
    timestamp: i64,
) -> Result<(), sqlx::Error> {
    println!(
        "Processing AmuletDropped: user={}, amulet_type={}, source={}",
        user, amulet_type, source
    );

    let user_str = user.to_string();
    let created_at = get_current_cst_time();

    // 1. Insert amulet drop history with type information
    crate::db::insert_amulet_drop_history_with_type(
        &pool,
        &user_str,
        amulet_type,
        &source,
        created_at,
    )
    .await?;

    // 2. No need to update user amulet collection stats - removed table

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
    let created_at = get_current_cst_time();

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

    // 2. No need to update user amulet collection stats - removed table

    Ok(())
}

/// Handle WishTowerUpdated event
async fn handle_wish_tower_updated(
    pool: &DbPool,
    user: solana_sdk::pubkey::Pubkey,
    wish_count: u32,
    level: u8,
    timestamp: i64,
) -> Result<(), sqlx::Error> {
    println!(
        "Processing WishTowerUpdated: user={}, wish_count={}, level={}",
        user, wish_count, level
    );

    let user_str = user.to_string();
    let updated_at = get_current_cst_time();

    // For now, we don't have a specific wish_tower table in the database
    // This event is mainly for tracking tower progress, but since the tower
    // state is stored on-chain, we might not need to store it in the database
    // unless we want to track historical tower states.

    // TODO: Consider adding a wish_towers table if we need to track tower progress

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
    let updated_at = get_current_cst_time();

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

/// Handle FortuneNFTMinted event
async fn handle_fortune_nft_minted(
    pool: &DbPool,
    user: solana_sdk::pubkey::Pubkey,
    fortune_nft_mint: solana_sdk::pubkey::Pubkey,
    fortune_result: String,
    merit_cost: u32,
    serial_number: u32,
    timestamp: i64,
) -> Result<(), sqlx::Error> {
    println!(
        "Processing FortuneNFTMinted: user={}, fortune_nft_mint={}, fortune_result={}, merit_cost={}, serial_number={}",
        user, fortune_nft_mint, fortune_result, merit_cost, serial_number
    );

    let user_str = user.to_string();
    let fortune_nft_mint_str = fortune_nft_mint.to_string();
    let created_at = get_current_cst_time();

    // 1. Insert fortune NFT mint history
    insert_fortune_nft_mint_history(
        &pool,
        &user_str,
        &fortune_nft_mint_str,
        &fortune_result,
        merit_cost as i32,
        serial_number as i32,
        created_at,
    )
    .await?;

    Ok(())
}
