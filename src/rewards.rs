/// Calculate rewards for donation based on level
pub fn calculate_donation_rewards(level: u8) -> (u64, u64) {
    // Match the exact rewards from product documentation
    match level {
        1 => (65, 1200),
        2 => (1300, 6300),
        3 => (14000, 30000),
        4 => (120000, 100000),
        _ => (0, 0),
    }
}

/// Calculate rewards for incense burning
pub fn calculate_incense_rewards(incense_type: u8, amount: u64) -> (u64, u64) {
    // Base rewards per incense type
    let (base_merit, base_incense_points) = match incense_type {
        1 => (5, 2),   // Basic incense
        2 => (12, 6),  // Premium incense
        3 => (25, 12), // Deluxe incense
        _ => (0, 0),
    };

    // Multiply by amount
    (base_merit * amount, base_incense_points * amount)
}

/// Calculate merit cost for fortune drawing
pub fn calculate_fortune_cost(is_free: bool) -> u64 {
    if is_free {
        0
    } else {
        50 // Base merit cost for fortune draw
    }
}

/// Check if fortune draw should be free based on user state
pub fn should_fortune_be_free(user_total_fortune_draws: u32) -> bool {
    // Every 10th draw is free, for example
    user_total_fortune_draws % 10 == 0
}
