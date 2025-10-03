pub mod models;

use chrono::{DateTime, Utc};
use sqlx::mysql::MySqlPool;
use sqlx::MySqlPool as SqlxMySqlPool;
use std::sync::Arc;

pub type DbPool = Arc<SqlxMySqlPool>;

pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let pool = MySqlPool::connect(database_url).await?;
    Ok(Arc::new(pool))
}

pub async fn init_database(pool: &DbPool) -> Result<(), sqlx::Error> {
    // Create tables one by one to avoid MySQL multi-statement issues

    // Global Stats table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS global_stats (
            id INT AUTO_INCREMENT PRIMARY KEY,
            total_merit BIGINT NOT NULL,
            total_incense_points BIGINT NOT NULL,
            total_donations_sol DOUBLE NOT NULL,
            total_users INT NOT NULL,
            total_wishes INT NOT NULL,
            updated_at DATETIME NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

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

    // Donation Leaderboard table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS donation_leaderboard (
            id INT AUTO_INCREMENT PRIMARY KEY,
            program_id VARCHAR(44) NOT NULL UNIQUE,
            top_donors TEXT NOT NULL,
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
            total_donated DOUBLE NOT NULL,
            donation_count INT NOT NULL,
            last_donation_at BIGINT,
            updated_at DATETIME NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE KEY unique_user_pubkey (user_pubkey)
        )"#,
    )
    .execute(pool.as_ref())
    .await?;

    // Incense Leaderboard table
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS incense_leaderboard (
            id INT AUTO_INCREMENT PRIMARY KEY,
            incense_type INT NOT NULL,
            top_users TEXT NOT NULL,
            updated_at DATETIME NOT NULL,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE KEY unique_incense_type (incense_type)
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
            has_buddha_nft BOOLEAN NOT NULL DEFAULT FALSE,
            has_medal_nft BOOLEAN NOT NULL DEFAULT FALSE,
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

    // Create indexes (ignore if already exists)
    let _ = sqlx::query(r#"CREATE INDEX idx_global_stats_updated_at ON global_stats(updated_at)"#)
        .execute(pool.as_ref())
        .await;

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

// ===== data handler =====

use crate::db::models::*;

/// upsert global stats
pub async fn upsert_global_stats(
    pool: &DbPool,
    total_merit: u64,
    total_incense_points: u64,
    total_donations_lamports: u64,
    total_users: u64,
    total_wishes: u64,
    updated_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let total_donations_sol = total_donations_lamports as f64 / 1_000_000_000.0;

    sqlx::query(
        r#"
        INSERT INTO global_stats (
            total_merit, total_incense_points, total_donations_sol,
            total_users, total_wishes, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(total_merit as i64)
    .bind(total_incense_points as i64)
    .bind(total_donations_sol)
    .bind(total_users as i32)
    .bind(total_wishes as i32)
    .bind(updated_at)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

pub async fn get_latest_global_stats(pool: &DbPool) -> Result<Option<GlobalStats>, sqlx::Error> {
    sqlx::query_as::<_, GlobalStats>(
        r#"
        SELECT * FROM global_stats
        ORDER BY updated_at DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn upsert_user_donation(
    pool: &DbPool,
    user_pubkey: &str,
    total_donated_lamports: u64,
    donation_count: u32,
    last_donation_at: Option<i64>,
    updated_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let total_donated_sol = total_donated_lamports as f64 / 1_000_000_000.0;

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
    .bind(total_donated_sol)
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

/// donation leaderboard
pub async fn get_donation_leaderboard(
    pool: &DbPool,
    limit: i32,
) -> Result<Vec<UserDonation>, sqlx::Error> {
    sqlx::query_as::<_, UserDonation>(
        r#"
        SELECT * FROM user_donations
        ORDER BY total_donated DESC
        LIMIT ?
        "#,
    )
    .bind(limit)
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

/// Atomically update global stats by donation event
pub async fn upsert_global_stats_by_donation(
    pool: &DbPool,
    merit_gained: u64,
    incense_points_gained: u64,
    donation_amount: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO global_stats (
            total_donations, total_donation_amount, total_merit_distributed, total_incense_points_distributed, updated_at, created_at
        ) VALUES (1, ?, ?, ?, NOW(), NOW())
        ON DUPLICATE KEY UPDATE
            total_donations = total_donations + 1,
            total_donation_amount = total_donation_amount + VALUES(total_donation_amount),
            total_merit_distributed = total_merit_distributed + VALUES(total_merit_distributed),
            total_incense_points_distributed = total_incense_points_distributed + VALUES(total_incense_points_distributed),
            updated_at = NOW()
        "#,
    )
    .bind(donation_amount as i64)
    .bind(merit_gained as i64)
    .bind(incense_points_gained as i64)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// Atomically update user state by rewards processed event
pub async fn upsert_user_state_by_rewards(
    pool: &DbPool,
    user_pubkey: &str,
    merit_reward: u64,
    incense_points_reward: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO user_states (
            user_pubkey, merit, incense_points, total_donation_amount, total_wish_count, total_fortune_draws, updated_at, created_at
        ) VALUES (?, ?, ?, 0, 0, 0, NOW(), NOW())
        ON DUPLICATE KEY UPDATE
            merit = merit + VALUES(merit),
            incense_points = incense_points + VALUES(incense_points),
            updated_at = NOW()
        "#,
    )
    .bind(user_pubkey)
    .bind(merit_reward as i64)
    .bind(incense_points_reward as i64)
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

/// Atomically update global stats by rewards processed event
pub async fn upsert_global_stats_by_rewards(
    pool: &DbPool,
    merit_reward: u64,
    incense_points_reward: u64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO global_stats (
            total_donations, total_donation_amount, total_merit_distributed, total_incense_points_distributed, updated_at, created_at
        ) VALUES (0, 0, ?, ?, NOW(), NOW())
        ON DUPLICATE KEY UPDATE
            total_merit_distributed = total_merit_distributed + VALUES(total_merit_distributed),
            total_incense_points_distributed = total_incense_points_distributed + VALUES(total_incense_points_distributed),
            updated_at = NOW()
        "#,
    )
    .bind(merit_reward as i64)
    .bind(incense_points_reward as i64)
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
        .bind(item.item_type as i32)
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
