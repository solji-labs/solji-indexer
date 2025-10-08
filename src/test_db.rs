use crate::db::{
    create_pool, get_aggregated_global_stats, get_parsed_incense_leaderboard_by_period,
    init_database, update_incense_leaderboard_all_periods,
};
use crate::utils::config::Config;

#[tokio::test]
async fn test_global_stats_storage() {
    // Load config
    let config = Config::from_env().expect("Failed to load config");

    // Create database connection pool
    let pool = create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");

    // Drop existing tables if they exist (to handle schema changes)
    let _ = sqlx::query("DROP TABLE IF EXISTS global_stats")
        .execute(pool.as_ref())
        .await;
    let _ = sqlx::query("DROP TABLE IF EXISTS temple_config")
        .execute(pool.as_ref())
        .await;
    let _ = sqlx::query("DROP TABLE IF EXISTS donation_leaderboard")
        .execute(pool.as_ref())
        .await;
    let _ = sqlx::query("DROP TABLE IF EXISTS user_donations")
        .execute(pool.as_ref())
        .await;
    let _ = sqlx::query("DROP TABLE IF EXISTS incense_leaderboard")
        .execute(pool.as_ref())
        .await;
    let _ = sqlx::query("DROP TABLE IF EXISTS wishes")
        .execute(pool.as_ref())
        .await;

    // Initialize database tables
    init_database(&pool)
        .await
        .expect("Failed to initialize database");

    println!("Database initialized successfully");

    // Mock GlobalStats data
    let total_merit = 1000u64;
    let total_incense_points = 5000u64;
    let total_donations_lamports = 10_000_000_000u64; // 10 SOL in lamports
    let total_users = 50u64;
    let total_wishes = 200u64;
    let updated_at = chrono::Utc::now();

    // Query aggregated global stats (from individual tables)
    let stats = get_aggregated_global_stats(&pool)
        .await
        .expect("Failed to get aggregated global stats");

    // Verify data
    assert_eq!(stats.total_merit, total_merit as i64);
    assert_eq!(stats.total_incense_points, total_incense_points as i64);
    assert_eq!(stats.total_users, total_users as i32);
    assert_eq!(stats.total_wishes, total_wishes as i32);

    // Verify updated_at is close to the expected time (allow for microsecond differences)
    let time_diff = (stats.updated_at - updated_at).num_milliseconds().abs();
    assert!(
        time_diff < 1000,
        "Time difference too large: {}ms",
        time_diff
    );

    // Verify SOL conversion
    let expected_sol = total_donations_lamports as f64 / 1_000_000_000.0;
    assert!((stats.total_donations_sol - expected_sol).abs() < 0.000001);

    println!("GlobalStats verification successful!");
    println!("Stored data:");
    println!("   Total Merit: {}", stats.total_merit);
    println!("   Total Incense Points: {}", stats.total_incense_points);
    println!("   Total Donations (SOL): {:.9}", stats.total_donations_sol);
    println!("   Total Users: {}", stats.total_users);
    println!("   Total Wishes: {}", stats.total_wishes);
    println!("   Updated At: {}", stats.updated_at);
    println!("   Created At: {}", stats.created_at);
}

#[tokio::test]
async fn test_incense_leaderboard() {
    // Load config
    let config = Config::from_env().expect("Failed to load config");

    // Create database connection pool
    let pool = create_pool(&config.database_url)
        .await
        .expect("Failed to create database pool");

    // Update leaderboard for all periods
    println!("Updating incense leaderboard...");
    update_incense_leaderboard_all_periods(&pool, chrono::Utc::now())
        .await
        .expect("Failed to update leaderboard");

    // Test getting leaderboard for different periods
    let periods = vec!["all", "daily", "weekly", "monthly"];

    for period in periods {
        println!("\n=== {} Leaderboard ===", period.to_uppercase());
        let leaderboard = get_parsed_incense_leaderboard_by_period(&pool, period)
            .await
            .expect("Failed to get leaderboard");

        if leaderboard.is_empty() {
            println!("No data for {} period", period);
        } else {
            println!("Found {} entries", leaderboard.len());
            for entry in leaderboard.iter().take(5) {
                // Show top 5
                println!(
                    "Rank {}: {} - {} points, {} burns",
                    entry.rank, entry.user_pubkey, entry.total_incense_points, entry.burn_count
                );
            }
        }
    }

    println!("\nIncense leaderboard test completed!");
}
