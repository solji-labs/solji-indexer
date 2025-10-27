pub mod models;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::MySqlPool as SqlxMySqlPool;
use std::sync::Arc;
use std::time::Duration;

pub type DbPool = Arc<SqlxMySqlPool>;

pub async fn create_pool(
    database_url: &str,
    max_open_conns: u32,
    max_idle_conns: u32,
    conn_max_lifetime: u64,
) -> Result<DbPool, sqlx::Error> {
    let pool = MySqlPoolOptions::new()
        .max_connections(max_open_conns)
        .min_connections(max_idle_conns)
        .max_lifetime(Duration::from_secs(conn_max_lifetime))
        .connect(database_url)
        .await?;

    Ok(Arc::new(pool))
}

pub async fn init_database(pool: &DbPool) -> Result<(), sqlx::Error> {
    // Create tables one by one to avoid MySQL multi-statement issues

    // Temple Config table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS temple_config (
            id INT AUTO_INCREMENT PRIMARY KEY,
            program_id VARCHAR(44) NOT NULL UNIQUE,
            incense_types TEXT NOT NULL,
            fortune_texts TEXT NOT NULL,
            updated_at DATETIME NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // User Donations table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS user_donations (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_pubkey VARCHAR(44) NOT NULL,
            total_donated BIGINT NOT NULL,
            donation_count INT NOT NULL,
            last_donation_at  DATETIME NOT NULL,
            updated_at DATETIME NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE KEY unique_user_pubkey (user_pubkey)
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // Donation History table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS donation_history (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_pubkey VARCHAR(44) NOT NULL,
            amount DECIMAL(20,10) NOT NULL,
            tier VARCHAR(20) NOT NULL,
            merit_gained INT NOT NULL,
            transaction_signature VARCHAR(88) NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX idx_user_pubkey (user_pubkey),
            INDEX idx_created_at (created_at)
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // User Donation Badges table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS user_donation_badges (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_pubkey VARCHAR(44) NOT NULL,
            tier VARCHAR(20) NOT NULL,
            badge_name VARCHAR(100) NOT NULL,
            earned_at DATETIME NOT NULL,
            total_donated DECIMAL(20,10) NOT NULL,
            nft_mint VARCHAR(44),
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX idx_user_pubkey (user_pubkey),
            INDEX idx_tier (tier),
            UNIQUE KEY unique_user_tier (user_pubkey, tier)
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // Incense Leaderboard table (global leaderboard, not per incense type)
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS incense_leaderboard (
            id INT AUTO_INCREMENT PRIMARY KEY,
            period_type VARCHAR(20) NOT NULL DEFAULT 'all', -- 'all', 'daily', 'weekly', 'monthly'
            top_users TEXT NOT NULL,
            updated_at DATETIME NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE KEY unique_period_type (period_type)
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // Wishes table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS wishes (
            id INT AUTO_INCREMENT PRIMARY KEY,
            wish_id BIGINT NOT NULL UNIQUE,
            user_pubkey VARCHAR(44) NOT NULL,
            content TEXT NOT NULL,
            likes INT NOT NULL DEFAULT 0,
            created_at DATETIME NOT NULL,
            updated_at DATETIME NOT NULL
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // User States table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS user_states (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_pubkey VARCHAR(44) NOT NULL UNIQUE,
            merit BIGINT NOT NULL DEFAULT 0,
            incense_points BIGINT NOT NULL DEFAULT 0,
            total_donation_amount BIGINT NOT NULL DEFAULT 0,
            total_wish_count INT NOT NULL DEFAULT 0,
            total_fortune_draws INT NOT NULL DEFAULT 0,
            pending_random_request_id VARCHAR(64),
            pending_amulets INT NOT NULL DEFAULT 0,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // Incense Burn History table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS incense_burn_history (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_pubkey VARCHAR(44) NOT NULL,
            incense_type INT NOT NULL,
            incense_amount INT NOT NULL,
            merit_gained BIGINT NOT NULL,
            incense_points_gained BIGINT NOT NULL,
            transaction_signature VARCHAR(88) NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX idx_user_pubkey (user_pubkey),
            INDEX idx_created_at (created_at)
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // Daily Incense Burn Count table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS daily_incense_burn_count (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_pubkey VARCHAR(44) NOT NULL,
            incense_type INT NOT NULL,
            burn_count INT NOT NULL DEFAULT 0,
            date DATE NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE KEY unique_user_incense_date (user_pubkey, incense_type, date),
            INDEX idx_user_pubkey_date (user_pubkey, date)
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // Fortune Draw History table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS fortune_draw_history (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_pubkey VARCHAR(44) NOT NULL,
            fortune_text TEXT NOT NULL,
            merit_cost BIGINT NOT NULL DEFAULT 0,
            is_free BOOLEAN NOT NULL DEFAULT TRUE,
            transaction_signature VARCHAR(88) NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX idx_user_pubkey (user_pubkey),
            INDEX idx_created_at (created_at)
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // Shop Config table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS shop_config (
            id INT AUTO_INCREMENT PRIMARY KEY,
            shop_config_pubkey VARCHAR(44) NOT NULL UNIQUE,
            temple_config_pubkey VARCHAR(44) NOT NULL,
            owner_pubkey VARCHAR(44) NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX idx_shop_config_pubkey (shop_config_pubkey)
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // Shop Items table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS shop_items (
            id INT AUTO_INCREMENT PRIMARY KEY,
            shop_config_id VARCHAR(44) NOT NULL,
            item_id INT NOT NULL,
            name VARCHAR(255) NOT NULL,
            description TEXT NOT NULL,
            price BIGINT NOT NULL,
            item_type INT NOT NULL,
            stock BIGINT NOT NULL,
            is_available BOOLEAN NOT NULL DEFAULT TRUE,
            merit BIGINT,
            incense_points BIGINT,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE KEY unique_shop_item (shop_config_id, item_id),
            INDEX idx_shop_config_id (shop_config_id)
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // Amulet Drop History table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS amulet_drop_history (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_pubkey VARCHAR(44) NOT NULL,
            amulet_type INT NOT NULL DEFAULT 0, -- 0: Fortune, 1: Protection, 2: Merit
            source VARCHAR(20) NOT NULL, -- 'burn_incense', 'draw_fortune', 'create_wish', 'purchase'
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX idx_user_pubkey (user_pubkey),
            INDEX idx_amulet_type (amulet_type),
            INDEX idx_source (source),
            INDEX idx_created_at (created_at)
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // User Amulet Collection table (removed - not needed for NFT minting logic)

    // Amulet Mint History table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS amulet_mint_history (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_pubkey VARCHAR(44) NOT NULL,
            amulet_mint VARCHAR(44) NOT NULL,
            source VARCHAR(20) NOT NULL, -- 'fortune' or 'wish'
            serial_number INT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX idx_user_pubkey (user_pubkey),
            INDEX idx_amulet_mint (amulet_mint),
            INDEX idx_source (source),
            INDEX idx_created_at (created_at)
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // Fortune NFT Mint History table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS fortune_nft_mint_history (
            id INT AUTO_INCREMENT PRIMARY KEY,
            user_pubkey VARCHAR(44) NOT NULL,
            fortune_nft_mint VARCHAR(44) NOT NULL,
            fortune_result VARCHAR(20) NOT NULL, -- 'Great Luck', 'Good Luck', etc.
            merit_cost INT NOT NULL DEFAULT 0,
            serial_number INT NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX idx_user_pubkey (user_pubkey),
            INDEX idx_fortune_nft_mint (fortune_nft_mint),
            INDEX idx_fortune_result (fortune_result),
            INDEX idx_created_at (created_at)
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // Temple Level table (removed - not needed for current implementation)

    // Create indexes (ignore if already exists)
    let _ = sqlx::query(
        r#"CREATE INDEX idx_user_donations_user_pubkey ON user_donations(user_pubkey)"#,
    )
    .execute(pool.as_ref())
    .await;

    let _ = sqlx::query(r#"CREATE INDEX idx_wishes_created_at ON wishes(created_at)"#)
        .execute(pool.as_ref())
        .await;

    let _ = sqlx::query(r#"CREATE INDEX idx_wishes_likes ON wishes(likes DESC)"#)
        .execute(pool.as_ref())
        .await;

    Ok(())
}

// ===== FORTUNE-RELATED DATABASE FUNCTIONS =====

/// Get fortune leaderboard (top users by fortune draws)
pub async fn get_fortune_leaderboard(
    pool: &DbPool,
    limit: i32,
) -> Result<Vec<FortuneLeaderboardEntry>, sqlx::Error> {
    let query = r#"
        SELECT
            user_pubkey,
            total_fortune_draws as total_draws
        FROM user_states
        WHERE total_fortune_draws > 0
        ORDER BY total_fortune_draws DESC
        LIMIT ?
        "#;

    let rows = sqlx::query_as::<_, (String, i32)>(query)
        .bind(limit)
        .fetch_all(pool.as_ref())
        .await?;

    let entries = rows
        .into_iter()
        .map(|(user_pubkey, total_draws)| FortuneLeaderboardEntry {
            user_pubkey,
            total_draws,
        })
        .collect();

    Ok(entries)
}

/// Get user's fortune statistics
pub async fn get_user_fortune_stats(
    pool: &DbPool,
    user_pubkey: &str,
) -> Result<UserFortuneStats, sqlx::Error> {
    // Get total fortune draws from fortune_draw_history table
    let total_draws = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM fortune_draw_history WHERE user_pubkey = ?",
    )
    .bind(user_pubkey)
    .fetch_one(pool.as_ref())
    .await?;

    // Get fortune NFT count
    let fortune_nft_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM fortune_nft_mint_history WHERE user_pubkey = ?",
    )
    .bind(user_pubkey)
    .fetch_one(pool.as_ref())
    .await?;

    // Get recent fortune draws (last 10)
    let recent_draws = get_user_fortune_draw_history(pool, user_pubkey, 10).await?;

    // Calculate rank
    let rank = sqlx::query_scalar::<_, Option<i64>>(
        r#"
        SELECT COUNT(DISTINCT fd.user_pubkey) + 1
        FROM fortune_draw_history fd
        WHERE (
            SELECT COUNT(*)
            FROM fortune_draw_history
            WHERE user_pubkey = fd.user_pubkey
        ) > (
            SELECT COUNT(*)
            FROM fortune_draw_history
            WHERE user_pubkey = ?
        )
        "#,
    )
    .bind(user_pubkey)
    .fetch_one(pool.as_ref())
    .await?;

    Ok(UserFortuneStats {
        user_pubkey: user_pubkey.to_string(),
        total_draws: total_draws as i32,
        fortune_nft_count: fortune_nft_count as i32,
        rank: rank.unwrap_or(0) as i32,
        recent_draws,
    })
}

/// Get fortune NFT mints for user
pub async fn get_user_fortune_nft_mints(
    pool: &DbPool,
    user_pubkey: &str,
    limit: i32,
) -> Result<Vec<FortuneNftMintHistory>, sqlx::Error> {
    sqlx::query_as::<_, FortuneNftMintHistory>(
        r#"
        SELECT * FROM fortune_nft_mint_history
        WHERE user_pubkey = ?
        ORDER BY created_at DESC
        LIMIT ?
        "#,
    )
    .bind(user_pubkey)
    .bind(limit)
    .fetch_all(pool.as_ref())
    .await
}

// ===== RESPONSE STRUCTS FOR FORTUNE =====

/// Fortune leaderboard entry
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FortuneLeaderboardEntry {
    pub user_pubkey: String,
    pub total_draws: i32,
}

/// User fortune statistics
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserFortuneStats {
    pub user_pubkey: String,
    pub total_draws: i32,
    pub fortune_nft_count: i32,
    pub rank: i32,
    pub recent_draws: Vec<FortuneDrawHistory>,
}

// ===== data handler =====

use crate::db::models::*;

/// Get total fortune NFTs count from fortune NFT mint history
async fn get_total_fortune_nfts(pool: &DbPool) -> Result<i32, sqlx::Error> {
    // Count total fortune NFTs minted from fortune_nft_mint_history table
    let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM fortune_nft_mint_history")
        .fetch_one(pool.as_ref())
        .await
        .unwrap_or(0);
    Ok(count as i32)
}

/// Get aggregated global stats from all tables
pub async fn get_aggregated_global_stats(pool: &DbPool) -> Result<GlobalStats, sqlx::Error> {
    // Get total users from user_states table
    let total_users_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM user_states")
        .fetch_one(pool.as_ref())
        .await?;

    // Get total wishes from wishes table
    let total_wishes_result = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM wishes")
        .fetch_one(pool.as_ref())
        .await?;

    // Get total donations and donation amount from user_donations table
    // Use CAST to force BIGINT type for SUM results
    let donation_count: i64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(donation_count), 0) AS SIGNED) FROM user_donations",
    )
    .fetch_one(pool.as_ref())
    .await?;

    let total_donated_lamports: i64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(total_donated), 0) AS SIGNED) FROM user_donations",
    )
    .fetch_one(pool.as_ref())
    .await?;

    // Get total merit and incense points from user_states table
    let user_stats = sqlx::query_as::<_, (i64, i64)>(
        "SELECT CAST(COALESCE(SUM(merit), 0) AS SIGNED), CAST(COALESCE(SUM(incense_points), 0) AS SIGNED) FROM user_states",
    )
    .fetch_one(pool.as_ref())
    .await?;

    // Get total merit and incense points distributed from incense_burn_history table
    let distributed_stats = sqlx::query_as::<_, (i64, i64)>(
        "SELECT CAST(COALESCE(SUM(merit_gained), 0) AS SIGNED), CAST(COALESCE(SUM(incense_points_gained), 0) AS SIGNED) FROM incense_burn_history"
    )
    .fetch_one(pool.as_ref())
    .await?;

    // Get total fortune draws from fortune_draw_history table
    let total_draw_fortune_result =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM fortune_draw_history")
            .fetch_one(pool.as_ref())
            .await?;

    // Get total donations SOL (convert from lamports to SOL)
    let total_donations_sol = total_donated_lamports as f64 / 1_000_000_000.0;

    // Get latest update time from any table (use a more robust query)
    let latest_update = match sqlx::query_scalar::<_, chrono::DateTime<chrono::Utc>>(
        "SELECT updated_at FROM user_states ORDER BY updated_at DESC LIMIT 1",
    )
    .fetch_optional(pool.as_ref())
    .await?
    {
        Some(timestamp) => timestamp,
        None => chrono::Utc::now(), // If no data, use current time
    };

    Ok(GlobalStats {
        id: 0, // This won't be used since we're aggregating
        total_merit: user_stats.0,
        total_incense_points: user_stats.1,
        total_donations_sol,
        total_users: total_users_result as i32,
        total_wishes: total_wishes_result as i32,
        total_donations: donation_count as i32,
        total_donation_amount: total_donated_lamports, // Already in lamports
        total_merit_distributed: distributed_stats.0,
        total_incense_points_distributed: distributed_stats.1,
        total_draw_fortune: total_draw_fortune_result as i32,
        total_fortune_nfts: get_total_fortune_nfts(pool).await.unwrap_or(0),
        updated_at: latest_update,
        created_at: chrono::Utc::now(),
    })
}

pub async fn upsert_user_donation(
    pool: &DbPool,
    user_pubkey: &str,
    total_donated_lamports: u64,
    donation_count: u32,
    last_donation_at: Option<DateTime<Utc>>,
    updated_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO user_donations (
            user_pubkey, total_donated, donation_count, last_donation_at, updated_at
        ) VALUES (?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            total_donated = VALUES(total_donated),
            donation_count = VALUES(donation_count),
            last_donation_at = VALUES(last_donation_at),
            updated_at = VALUES(updated_at)
        "#,
    )
    .bind(user_pubkey)
    .bind(total_donated_lamports as i64)
    .bind(donation_count as i32)
    .bind(last_donation_at)
    .bind(updated_at)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

pub async fn upsert_wish(
    pool: &DbPool,
    wish_id: u64,
    user_pubkey: &str,
    content: &str,
    likes: u32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO wishes (
            wish_id, user_pubkey, content, likes, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            likes = VALUES(likes),
            updated_at = VALUES(updated_at)
        "#,
    )
    .bind(wish_id as i64)
    .bind(user_pubkey)
    .bind(content)
    .bind(likes as i32)
    .bind(created_at)
    .bind(updated_at)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// wish list
pub async fn get_wishes(pool: &DbPool, limit: i32, offset: i32) -> Result<Vec<Wish>, sqlx::Error> {
    sqlx::query_as::<_, Wish>(
        r#"
        SELECT * FROM wishes
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.as_ref())
    .await
}

/// upsert user state
pub async fn upsert_user_state(
    pool: &DbPool,
    user_pubkey: &str,
    merit: i64,
    incense_points: i64,
    total_donation_amount: i64,
    total_wish_count: i32,
    total_fortune_draws: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO user_states (
            user_pubkey, merit, incense_points, total_donation_amount, total_wish_count, total_fortune_draws, updated_at, created_at
        ) VALUES (?, ?, ?, ?, ?, ?, NOW(), NOW())
        ON DUPLICATE KEY UPDATE
            merit = VALUES(merit),
            incense_points = VALUES(incense_points),
            total_donation_amount = VALUES(total_donation_amount),
            total_wish_count = VALUES(total_wish_count),
            total_fortune_draws = VALUES(total_fortune_draws),
            updated_at = NOW()
        "#,
    )
    .bind(user_pubkey)
    .bind(merit)
    .bind(incense_points)
    .bind(total_donation_amount)
    .bind(total_wish_count)
    .bind(total_fortune_draws)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// Atomically update user state by donation event (incremental update)
pub async fn upsert_user_state_by_donation(
    pool: &DbPool,
    user_pubkey: &str,
    merit_gained: u64,
    incense_points_gained: u64,
    donation_amount: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO user_states (
            user_pubkey, merit, incense_points, total_donation_amount, total_wish_count, total_fortune_draws, updated_at, created_at
        ) VALUES (?, ?, ?, ?, 0, 0, NOW(), NOW())
        ON DUPLICATE KEY UPDATE
            merit = merit + VALUES(merit),
            incense_points = incense_points + VALUES(incense_points),
            total_donation_amount = total_donation_amount + VALUES(total_donation_amount),
            updated_at = NOW()
        "#,
    )
    .bind(user_pubkey)
    .bind(merit_gained as i64)
    .bind(incense_points_gained as i64)
    .bind(donation_amount as i64)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// insert incense burn history
pub async fn insert_incense_burn_history(
    pool: &DbPool,
    user_pubkey: &str,
    incense_type: i32,
    incense_amount: i32,
    merit_gained: i64,
    incense_points_gained: i64,
    transaction_signature: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO incense_burn_history (
            user_pubkey, incense_type, incense_amount,
            merit_gained, incense_points_gained, transaction_signature
        ) VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_pubkey)
    .bind(incense_type)
    .bind(incense_amount)
    .bind(merit_gained)
    .bind(incense_points_gained)
    .bind(transaction_signature)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// insert fortune draw history
pub async fn insert_fortune_draw_history(
    pool: &DbPool,
    user_pubkey: &str,
    fortune_text: &str,
    merit_cost: i64,
    is_free: bool,
    transaction_signature: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO fortune_draw_history (
            user_pubkey, fortune_text, merit_cost, is_free, transaction_signature
        ) VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_pubkey)
    .bind(fortune_text)
    .bind(merit_cost)
    .bind(is_free)
    .bind(transaction_signature)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// insert fortune NFT mint history
pub async fn insert_fortune_nft_mint_history(
    pool: &DbPool,
    user_pubkey: &str,
    fortune_nft_mint: &str,
    fortune_result: &str,
    merit_cost: i32,
    serial_number: i32,
    created_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO fortune_nft_mint_history (
            user_pubkey, fortune_nft_mint, fortune_result, merit_cost, serial_number, created_at
        ) VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_pubkey)
    .bind(fortune_nft_mint)
    .bind(fortune_result)
    .bind(merit_cost)
    .bind(serial_number)
    .bind(created_at)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// get user incense burn history
pub async fn get_user_incense_burn_history(
    pool: &DbPool,
    user_pubkey: &str,
    limit: i32,
) -> Result<Vec<IncenseBurnHistory>, sqlx::Error> {
    sqlx::query_as::<_, IncenseBurnHistory>(
        r#"
        SELECT * FROM incense_burn_history
        WHERE user_pubkey = ?
        ORDER BY created_at DESC
        LIMIT ?
        "#,
    )
    .bind(user_pubkey)
    .bind(limit)
    .fetch_all(pool.as_ref())
    .await
}

/// get user fortune draw history
pub async fn get_user_fortune_draw_history(
    pool: &DbPool,
    user_pubkey: &str,
    limit: i32,
) -> Result<Vec<FortuneDrawHistory>, sqlx::Error> {
    sqlx::query_as::<_, FortuneDrawHistory>(
        r#"
        SELECT * FROM fortune_draw_history
        WHERE user_pubkey = ?
        ORDER BY created_at DESC
        LIMIT ?
        "#,
    )
    .bind(user_pubkey)
    .bind(limit)
    .fetch_all(pool.as_ref())
    .await
}

/// upsert shop config
pub async fn upsert_shop_config(
    pool: &DbPool,
    shop_config_pubkey: &str,
    temple_config_pubkey: &str,
    owner_pubkey: &str,
    updated_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO shop_config (
            shop_config_pubkey, temple_config_pubkey, owner_pubkey, updated_at
        ) VALUES (?, ?, ?, ?)
        ON DUPLICATE KEY UPDATE
            temple_config_pubkey = VALUES(temple_config_pubkey),
            owner_pubkey = VALUES(owner_pubkey),
            updated_at = VALUES(updated_at)
        "#,
    )
    .bind(shop_config_pubkey)
    .bind(temple_config_pubkey)
    .bind(owner_pubkey)
    .bind(updated_at)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// sync shop items (delete existing and insert new ones)
pub async fn sync_shop_items(
    pool: &DbPool,
    shop_config_id: &str,
    shop_items: &[crate::events::ShopItem],
    updated_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    // Start transaction
    let mut tx = pool.begin().await?;

    // Delete existing items for this shop config
    sqlx::query("DELETE FROM shop_items WHERE shop_config_id = ?")
        .bind(shop_config_id)
        .execute(&mut *tx)
        .await?;

    // Insert new items
    for item in shop_items {
        let (merit, incense_points) = match &item.incense_config {
            Some(config) => (
                Some(config.merit as i64),
                Some(config.incense_points as i64),
            ),
            None => (None, None),
        };

        sqlx::query(
            r#"
            INSERT INTO shop_items (
                shop_config_id, item_id, name, description, price, item_type,
                stock, is_available, merit, incense_points, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(shop_config_id)
        .bind(item.id as i32)
        .bind(&item.name)
        .bind(&item.description)
        .bind(item.price as i64)
        .bind(item.item_type.clone() as i32)
        .bind(item.stock as i64)
        .bind(item.is_available)
        .bind(merit)
        .bind(incense_points)
        .bind(updated_at)
        .execute(&mut *tx)
        .await?;
    }

    // Commit transaction
    tx.commit().await?;

    Ok(())
}

/// get donation leaderboard data directly from user_donations table
pub async fn get_donation_leaderboard(
    pool: &DbPool,
    limit: usize,
    offset: usize,
) -> Result<Vec<DonationLeaderboardEntry>, sqlx::Error> {
    let query = r#"
        SELECT
            user_pubkey,
            total_donated
        FROM user_donations
        ORDER BY total_donated DESC
        LIMIT ? OFFSET ?
        "#;

    let donors = sqlx::query_as::<_, (String, i64)>(query)
        .bind(limit as i32)
        .bind(offset as i32)
        .fetch_all(pool.as_ref())
        .await?;

    let entries: Vec<DonationLeaderboardEntry> = donors
        .into_iter()
        .enumerate()
        .map(
            |(index, (user_pubkey, total_donated_lamports))| DonationLeaderboardEntry {
                rank: offset + index + 1,
                user_pubkey,
                total_donated: total_donated_lamports as f64 / 1_000_000_000.0, // Convert lamports to SOL
            },
        )
        .collect();

    Ok(entries)
}

/// check if user is in top 10000 donors
pub async fn check_user_in_top_10000_donors(
    pool: &DbPool,
    user_pubkey: &str,
) -> Result<bool, sqlx::Error> {
    // Check if user is within top 10000 by finding their rank
    let result = sqlx::query_scalar::<_, Option<i64>>(
        r#"
        SELECT COUNT(*) + 1
        FROM user_donations
        WHERE total_donated > (
            SELECT total_donated
            FROM user_donations
            WHERE user_pubkey = ?
        )
        "#,
    )
    .bind(user_pubkey)
    .fetch_one(pool.as_ref())
    .await?;

    match result {
        Some(rank) => Ok(rank <= 10000),
        None => Ok(false), // User not found
    }
}

/// update incense leaderboard for all periods
pub async fn update_incense_leaderboard_all_periods(
    pool: &DbPool,
    updated_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let periods = vec!["all", "daily", "weekly", "monthly"];

    for period in periods {
        update_incense_leaderboard_by_period(pool, period, updated_at).await?;
    }

    Ok(())
}

/// update incense leaderboard for a specific period
pub async fn update_incense_leaderboard_by_period(
    pool: &DbPool,
    period: &str,
    updated_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let date_condition = match period {
        "daily" => "DATE(created_at) = CURDATE()",
        "weekly" => "YEARWEEK(created_at, 1) = YEARWEEK(CURDATE(), 1)",
        "monthly" => "DATE_FORMAT(created_at, '%Y-%m') = DATE_FORMAT(CURDATE(), '%Y-%m')",
        _ => "1=1", // No filter for "all"
    };

    // Calculate top 100 users by total incense_points_gained and burn count
    let query = format!(
        r#"
        SELECT
            user_pubkey,
            CAST(SUM(incense_points_gained) AS SIGNED) as total_incense_points,
            CAST(COUNT(*) AS SIGNED) as burn_count
        FROM incense_burn_history
        WHERE {}
        GROUP BY user_pubkey
        ORDER BY total_incense_points DESC, burn_count DESC
        LIMIT 100
        "#,
        date_condition
    );

    let top_users = sqlx::query_as::<_, (String, i64, i64)>(&query)
        .fetch_all(pool.as_ref())
        .await?;

    // Convert to JSON format
    let leaderboard_data: Vec<serde_json::Value> = top_users
        .into_iter()
        .enumerate()
        .map(|(rank, (user_pubkey, total_incense_points, burn_count))| {
            serde_json::json!({
                "rank": rank + 1,
                "user_pubkey": user_pubkey,
                "total_incense_points": total_incense_points,
                "burn_count": burn_count
            })
        })
        .collect();

    let top_users_json = serde_json::to_string(&leaderboard_data)
        .map_err(|e| sqlx::Error::Protocol(format!("JSON serialization error: {}", e)))?;

    // Update or insert leaderboard
    sqlx::query(
        r#"
        INSERT INTO incense_leaderboard (
            period_type, top_users, updated_at
        ) VALUES (?, ?, ?)
        ON DUPLICATE KEY UPDATE
            top_users = VALUES(top_users),
            updated_at = VALUES(updated_at)
        "#,
    )
    .bind(period)
    .bind(top_users_json)
    .bind(updated_at)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// get incense leaderboard by period
pub async fn get_incense_leaderboard_by_period(
    pool: &DbPool,
    period: &str,
) -> Result<Option<IncenseLeaderboard>, sqlx::Error> {
    sqlx::query_as::<_, IncenseLeaderboard>(
        r#"
        SELECT * FROM incense_leaderboard
        WHERE period_type = ?
        "#,
    )
    .bind(period)
    .fetch_optional(pool.as_ref())
    .await
}

/// API response struct for leaderboard entries
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct LeaderboardEntry {
    pub rank: usize,
    pub user_pubkey: String,
    pub total_incense_points: i64,
    pub burn_count: i64,
}

/// API response struct for donation leaderboard entries
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DonationLeaderboardEntry {
    pub rank: usize,
    pub user_pubkey: String,
    pub total_donated: f64,
}

/// get parsed leaderboard data by period
pub async fn get_parsed_incense_leaderboard_by_period(
    pool: &DbPool,
    period: &str,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    if let Some(leaderboard) = get_incense_leaderboard_by_period(pool, period).await? {
        match serde_json::from_str(&leaderboard.top_users) {
            Ok(entries) => Ok(entries),
            Err(_) => Ok(vec![]), // Return empty vec on parse error
        }
    } else {
        Ok(vec![])
    }
}

/// insert amulet drop history
pub async fn insert_amulet_drop_history(
    pool: &DbPool,
    user_pubkey: &str,
    source: &str,
    created_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO amulet_drop_history (
            user_pubkey, amulet_type, source, created_at
        ) VALUES (?, 0, ?, ?)
        "#,
    )
    .bind(user_pubkey)
    .bind(source)
    .bind(created_at)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// insert amulet drop history with type information
pub async fn insert_amulet_drop_history_with_type(
    pool: &DbPool,
    user_pubkey: &str,
    amulet_type: u8,
    source: &str,
    created_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO amulet_drop_history (
            user_pubkey, amulet_type, source, created_at
        ) VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(user_pubkey)
    .bind(amulet_type as i32)
    .bind(source)
    .bind(created_at)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// get user pending amulets
pub async fn get_user_pending_amulets(
    pool: &DbPool,
    user_pubkey: &str,
) -> Result<Vec<AmuletDropHistory>, sqlx::Error> {
    sqlx::query_as::<_, AmuletDropHistory>(
        r#"
        SELECT * FROM amulet_drop_history
        WHERE user_pubkey = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_pubkey)
    .fetch_all(pool.as_ref())
    .await
}

/// get user owned amulets
pub async fn get_user_owned_amulets(
    pool: &DbPool,
    user_pubkey: &str,
) -> Result<Vec<AmuletMintHistory>, sqlx::Error> {
    sqlx::query_as::<_, AmuletMintHistory>(
        r#"
        SELECT * FROM amulet_mint_history
        WHERE user_pubkey = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_pubkey)
    .fetch_all(pool.as_ref())
    .await
}

// Removed increment_user_amulet_stats function - table no longer exists

/// insert amulet mint history
pub async fn insert_amulet_mint_history(
    pool: &DbPool,
    user_pubkey: &str,
    amulet_mint: &str,
    source: &str,
    serial_number: u32,
    created_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO amulet_mint_history (
            user_pubkey, amulet_mint, source, serial_number, created_at
        ) VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_pubkey)
    .bind(amulet_mint)
    .bind(source)
    .bind(serial_number as i32)
    .bind(created_at)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

// Removed decrement_user_pending_amulets function - table no longer exists

/// get shop items for a shop config
pub async fn get_shop_items(
    pool: &DbPool,
    shop_config_id: &str,
) -> Result<Vec<ShopItem>, sqlx::Error> {
    sqlx::query_as::<_, ShopItem>(
        r#"
        SELECT * FROM shop_items
        WHERE shop_config_id = ?
        ORDER BY item_id
        "#,
    )
    .bind(shop_config_id)
    .fetch_all(pool.as_ref())
    .await
}

/// upsert user state NFT ownership (removed - NFT ownership now checked via ATA)

/// increment global stats NFT count (removed - NFT ownership now checked via ATA)

/// increment user fortune draws count
pub async fn increment_user_fortune_draws(
    pool: &DbPool,
    user_pubkey: &str,
    updated_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO user_states (
            user_pubkey, total_fortune_draws, updated_at, created_at
        ) VALUES (?, 1, ?, NOW())
        ON DUPLICATE KEY UPDATE
            total_fortune_draws = total_fortune_draws + 1,
            updated_at = VALUES(updated_at)
        "#,
    )
    .bind(user_pubkey)
    .bind(updated_at)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// check if user can burn incense (daily limit check)
pub async fn check_daily_incense_limit(
    pool: &DbPool,
    user_pubkey: &str,
    incense_type: i32,
    amount: i32,
) -> Result<bool, sqlx::Error> {
    let today = chrono::Utc::now().date_naive();

    // Get current burn count for today
    let result = sqlx::query_as::<_, DailyIncenseBurnCount>(
        r#"
        SELECT * FROM daily_incense_burn_count
        WHERE user_pubkey = ? AND incense_type = ? AND date = ?
        "#,
    )
    .bind(user_pubkey)
    .bind(incense_type)
    .bind(today)
    .fetch_optional(pool.as_ref())
    .await?;

    let current_count = result.map(|r| r.burn_count).unwrap_or(0);
    let max_daily_limit = 10; // Maximum 10 burns per incense type per day

    Ok(current_count + amount <= max_daily_limit)
}

/// update daily incense burn count
pub async fn update_daily_incense_burn_count(
    pool: &DbPool,
    user_pubkey: &str,
    incense_type: i32,
    amount: i32,
) -> Result<(), sqlx::Error> {
    let today = chrono::Utc::now().date_naive();

    sqlx::query(
        r#"
        INSERT INTO daily_incense_burn_count (
            user_pubkey, incense_type, burn_count, date, updated_at
        ) VALUES (?, ?, ?, ?, NOW())
        ON DUPLICATE KEY UPDATE
            burn_count = burn_count + VALUES(burn_count),
            updated_at = NOW()
        "#,
    )
    .bind(user_pubkey)
    .bind(incense_type)
    .bind(amount)
    .bind(today)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// update user incense balance and add incense burn history
pub async fn update_user_incense_and_history(
    pool: &DbPool,
    user_pubkey: &str,
    incense_type: i32,
    incense_amount: i32,
    merit_gained: i64,
    incense_points_gained: i64,
    transaction_signature: &str,
    updated_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    // Start transaction
    let mut tx = pool.begin().await?;

    // 1. Insert incense burn history
    sqlx::query(
        r#"
        INSERT INTO incense_burn_history (
            user_pubkey, incense_type, incense_amount,
            merit_gained, incense_points_gained, transaction_signature
        ) VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_pubkey)
    .bind(incense_type)
    .bind(incense_amount)
    .bind(merit_gained)
    .bind(incense_points_gained)
    .bind(transaction_signature)
    .execute(&mut *tx)
    .await?;

    // 2. Update daily burn count
    sqlx::query(
        r#"
        INSERT INTO daily_incense_burn_count (
            user_pubkey, incense_type, burn_count, date, updated_at
        ) VALUES (?, ?, ?, CURDATE(), NOW())
        ON DUPLICATE KEY UPDATE
            burn_count = burn_count + VALUES(burn_count),
            updated_at = NOW()
        "#,
    )
    .bind(user_pubkey)
    .bind(incense_type)
    .bind(incense_amount)
    .execute(&mut *tx)
    .await?;

    // 3. Update user state with gained values
    sqlx::query(
        r#"
        INSERT INTO user_states (
            user_pubkey, merit, incense_points, updated_at, created_at
        ) VALUES (?, ?, ?, ?, NOW())
        ON DUPLICATE KEY UPDATE
            merit = merit + VALUES(merit),
            incense_points = incense_points + VALUES(incense_points),
            updated_at = VALUES(updated_at)
        "#,
    )
    .bind(user_pubkey)
    .bind(merit_gained)
    .bind(incense_points_gained)
    .bind(updated_at)
    .execute(&mut *tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    Ok(())
}

// ===== WISH-RELATED DATABASE FUNCTIONS =====

/// Get user's wishes
pub async fn get_user_wishes(
    pool: &DbPool,
    user_pubkey: &str,
    limit: i32,
    offset: i32,
) -> Result<Vec<Wish>, sqlx::Error> {
    sqlx::query_as::<_, Wish>(
        r#"
        SELECT * FROM wishes
        WHERE user_pubkey = ?
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(user_pubkey)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.as_ref())
    .await
}

/// Get user's daily wish count (today)
pub async fn get_user_daily_wish_count(
    pool: &DbPool,
    user_pubkey: &str,
) -> Result<i32, sqlx::Error> {
    let today = chrono::Utc::now().date_naive();

    let result = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT COUNT(*) FROM wishes
        WHERE user_pubkey = ? AND DATE(created_at) = ?
        "#,
    )
    .bind(user_pubkey)
    .bind(today)
    .fetch_one(pool.as_ref())
    .await?;

    Ok(result)
}

/// Get user's wish tower stats (calculate level based on new requirements)
pub async fn get_user_wish_tower_stats(
    pool: &DbPool,
    user_pubkey: &str,
) -> Result<WishTowerStats, sqlx::Error> {
    // Get total wish count for user
    let total_wishes =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM wishes WHERE user_pubkey = ?")
            .bind(user_pubkey)
            .fetch_one(pool.as_ref())
            .await?;

    // Calculate level based on new requirements:
    // Level 0: 0 wishes (铸造获得)
    // Level 1: 10 wishes
    // Level 2: 50 wishes
    // Level 3: 200 wishes
    // Level 4: 500 wishes
    let level = if total_wishes >= 500 {
        4 // 圆满塔
    } else if total_wishes >= 200 {
        3 // 宏愿塔
    } else if total_wishes >= 50 {
        2 // 精进塔
    } else if total_wishes >= 10 {
        1 // 基础塔
    } else {
        0 // 种子塔
    };

    // Get latest wish time as last_updated
    let last_updated = sqlx::query_scalar::<_, DateTime<Utc>>(
        "SELECT created_at FROM wishes WHERE user_pubkey = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(user_pubkey)
    .fetch_optional(pool.as_ref())
    .await?;

    Ok(WishTowerStats {
        user_pubkey: user_pubkey.to_string(),
        total_wishes: total_wishes as i32,
        level: level as i32,
        last_updated,
    })
}

/// Like a wish (increment likes count)
pub async fn like_wish_by_id(pool: &DbPool, wish_id: i64) -> Result<i32, sqlx::Error> {
    // Update likes count and return new count
    let result = sqlx::query_scalar::<_, i32>(
        r#"
        UPDATE wishes
        SET likes = likes + 1, updated_at = NOW()
        WHERE wish_id = ?
        "#,
    )
    .bind(wish_id)
    .fetch_one(pool.as_ref())
    .await?;

    Ok(result)
}

/// Get public wishes (non-anonymous) with pagination
pub async fn get_public_wishes(
    pool: &DbPool,
    limit: i32,
    offset: i32,
) -> Result<Vec<Wish>, sqlx::Error> {
    // Note: Current schema doesn't have is_anonymous field
    // For now, return all wishes (assuming all are public)
    // TODO: Add is_anonymous field to wishes table when available
    sqlx::query_as::<_, Wish>(
        r#"
        SELECT * FROM wishes
        ORDER BY created_at DESC
        LIMIT ? OFFSET ?
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.as_ref())
    .await
}

/// Get wish by ID
pub async fn get_wish_by_id(pool: &DbPool, wish_id: i64) -> Result<Option<Wish>, sqlx::Error> {
    sqlx::query_as::<_, Wish>(
        r#"
        SELECT * FROM wishes
        WHERE wish_id = ?
        "#,
    )
    .bind(wish_id)
    .fetch_optional(pool.as_ref())
    .await
}

// ===== INCENSE-RELATED DATABASE FUNCTIONS =====

/// Get user's daily incense burn count for all incense types
pub async fn get_user_incense_burn_count(
    pool: &DbPool,
    user_pubkey: &str,
) -> Result<Vec<DailyIncenseBurnCount>, sqlx::Error> {
    sqlx::query_as::<_, DailyIncenseBurnCount>(
        r#"
        SELECT * FROM daily_incense_burn_count
        WHERE user_pubkey = ? AND date = CURDATE()
        ORDER BY incense_type
        "#,
    )
    .bind(user_pubkey)
    .fetch_all(pool.as_ref())
    .await
}

/// Get user's incense NFTs (placeholder - NFT ownership tracked via ATA)
pub async fn get_user_incense_nfts(
    pool: &DbPool,
    _user_pubkey: &str,
) -> Result<Vec<serde_json::Value>, sqlx::Error> {
    // For now, return empty array as NFT ownership is checked via ATA
    // TODO: Implement proper NFT tracking if needed
    Ok(vec![])
}

// ===== RESPONSE STRUCTS =====

/// Wish tower stats response
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct WishTowerStats {
    pub user_pubkey: String,
    pub total_wishes: i32,
    pub level: i32,
    pub last_updated: Option<DateTime<Utc>>,
}

// ===== DONATION-RELATED DATABASE FUNCTIONS =====

/// Insert donation history record
pub async fn insert_donation_history(
    pool: &DbPool,
    user_pubkey: &str,
    amount_sol: f64,
    tier: &str,
    merit_gained: i32,
    transaction_signature: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO donation_history (
            user_pubkey, amount, tier, merit_gained, transaction_signature
        ) VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_pubkey)
    .bind(amount_sol)
    .bind(tier)
    .bind(merit_gained)
    .bind(transaction_signature)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// Get user's donation history
pub async fn get_user_donation_history(
    pool: &DbPool,
    user_pubkey: &str,
    limit: i32,
) -> Result<Vec<DonationHistory>, sqlx::Error> {
    sqlx::query_as::<_, DonationHistory>(
        r#"
        SELECT * FROM donation_history
        WHERE user_pubkey = ?
        ORDER BY created_at DESC
        LIMIT ?
        "#,
    )
    .bind(user_pubkey)
    .bind(limit)
    .fetch_all(pool.as_ref())
    .await
}

/// Upsert user donation badge
pub async fn upsert_user_donation_badge(
    pool: &DbPool,
    user_pubkey: &str,
    tier: &str,
    badge_name: &str,
    total_donated: f64,
    nft_mint: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO user_donation_badges (
            user_pubkey, tier, badge_name, earned_at, total_donated, nft_mint
        ) VALUES (?, ?, ?, NOW(), ?, ?)
        ON DUPLICATE KEY UPDATE
            badge_name = VALUES(badge_name),
            total_donated = VALUES(total_donated),
            nft_mint = VALUES(nft_mint)
        "#,
    )
    .bind(user_pubkey)
    .bind(tier)
    .bind(badge_name)
    .bind(total_donated)
    .bind(nft_mint)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// Get user's donation badges
pub async fn get_user_donation_badges(
    pool: &DbPool,
    user_pubkey: &str,
) -> Result<Vec<UserDonationBadge>, sqlx::Error> {
    sqlx::query_as::<_, UserDonationBadge>(
        r#"
        SELECT * FROM user_donation_badges
        WHERE user_pubkey = ?
        ORDER BY earned_at DESC
        "#,
    )
    .bind(user_pubkey)
    .fetch_all(pool.as_ref())
    .await
}

/// Process donation transaction (insert history and update badges)
pub async fn process_donation_transaction(
    pool: &DbPool,
    user_pubkey: &str,
    amount_sol: f64,
    tier: &str,
    merit_gained: i32,
    transaction_signature: &str,
) -> Result<(), sqlx::Error> {
    // Start transaction
    let mut tx = pool.begin().await?;

    // 1. Insert donation history
    sqlx::query(
        r#"
        INSERT INTO donation_history (
            user_pubkey, amount, tier, merit_gained, transaction_signature
        ) VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_pubkey)
    .bind(amount_sol)
    .bind(tier)
    .bind(merit_gained)
    .bind(transaction_signature)
    .execute(&mut *tx)
    .await?;

    // 2. Update user donation stats
    let total_donated_lamports = (amount_sol * 1_000_000_000.0) as i64;
    sqlx::query(
        r#"
        INSERT INTO user_donations (
            user_pubkey, total_donated, donation_count, last_donation_at, updated_at
        ) VALUES (?, ?, 1, NOW(), NOW())
        ON DUPLICATE KEY UPDATE
            total_donated = total_donated + VALUES(total_donated),
            donation_count = donation_count + 1,
            last_donation_at = NOW(),
            updated_at = NOW()
        "#,
    )
    .bind(user_pubkey)
    .bind(total_donated_lamports)
    .execute(&mut *tx)
    .await?;

    // 3. Update or insert donation badge
    let badge_name = match tier {
        "bronze" => "Bronze Merit Badge",
        "silver" => "Silver Progress Badge",
        "gold" => "Gold Guardian Badge",
        "supreme" => "Supreme Dragon Badge",
        _ => "Unknown Badge",
    };

    // Get current total donated for this user
    let current_total: Option<f64> = sqlx::query_scalar(
        "SELECT total_donated / 1000000000.0 FROM user_donations WHERE user_pubkey = ?",
    )
    .bind(user_pubkey)
    .fetch_optional(&mut *tx)
    .await?;

    if let Some(total_donated) = current_total {
        sqlx::query(
            r#"
            INSERT INTO user_donation_badges (
                user_pubkey, tier, badge_name, earned_at, total_donated
            ) VALUES (?, ?, ?, NOW(), ?)
            ON DUPLICATE KEY UPDATE
                badge_name = VALUES(badge_name),
                total_donated = VALUES(total_donated)
            "#,
        )
        .bind(user_pubkey)
        .bind(tier)
        .bind(badge_name)
        .bind(total_donated)
        .execute(&mut *tx)
        .await?;
    }

    // 4. Update user state with merit points
    sqlx::query(
        r#"
        INSERT INTO user_states (
            user_pubkey, merit, updated_at, created_at
        ) VALUES (?, ?, NOW(), NOW())
        ON DUPLICATE KEY UPDATE
            merit = merit + VALUES(merit),
            updated_at = NOW()
        "#,
    )
    .bind(user_pubkey)
    .bind(merit_gained as i64)
    .execute(&mut *tx)
    .await?;

    // Commit transaction
    tx.commit().await?;

    Ok(())
}
