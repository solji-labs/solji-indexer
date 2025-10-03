use anchor_lang::prelude::*;
use base64::{engine::general_purpose, Engine};
use solana_sdk::pubkey::Pubkey;

use crate::events::ProgramEvent;
use temple::state::event::{
    DonationCompleted, DonationNFTMinted, FortuneDrawn, RewardsProcessed, WishCreated,
};

// Event discriminators from IDL
const DONATION_COMPLETED_DISCRIMINATOR: [u8; 8] = [34, 178, 117, 6, 39, 189, 241, 48];
const DONATION_NFT_MINTED_DISCRIMINATOR: [u8; 8] = [142, 88, 211, 148, 62, 90, 172, 20];
const FORTUNE_DRAWN_DISCRIMINATOR: [u8; 8] = [134, 252, 88, 211, 24, 112, 209, 240];
const REWARDS_PROCESSED_DISCRIMINATOR: [u8; 8] = [217, 74, 206, 32, 228, 181, 17, 146];
const WISH_CREATED_DISCRIMINATOR: [u8; 8] = [225, 167, 37, 207, 75, 1, 226, 130];

const EVENT_PREFIX: &str = "Program log: ";
const ANCHOR_EVENT_LABEL: &str = "Event:";

/// Core function: Try to parse an Anchor event from a log string
pub fn try_parse_event(log_str: &str) -> Option<ProgramEvent> {
    // 1. Find event prefix
    let event_log = log_str.strip_prefix(EVENT_PREFIX)?;
    let base64_data = event_log.strip_prefix(ANCHOR_EVENT_LABEL)?;

    // 2. Base64 decode
    let decoded_bytes = match general_purpose::STANDARD.decode(base64_data.trim()) {
        Ok(bytes) => bytes,
        Err(_) => return None,
    };

    // Ensure data is long enough (8 bytes discriminator)
    if decoded_bytes.len() < 8 {
        return None;
    }

    // Extract discriminator
    let discriminator = &decoded_bytes[0..8];

    // 3. Match discriminator and deserialize (skip the 8-byte discriminator)
    if discriminator == &DONATION_COMPLETED_DISCRIMINATOR {
        if let Ok(event) = DonationCompleted::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::DonationCompleted {
                user: Pubkey::new_from_array(event.user.to_bytes()),
                amount: event.amount,
                total_donated: event.total_donated,
                level: event.level,
                timestamp: event.timestamp,
            })
        } else {
            None
        }
    } else if discriminator == &DONATION_NFT_MINTED_DISCRIMINATOR {
        if let Ok(event) = DonationNFTMinted::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::DonationNFTMinted {
                user: Pubkey::new_from_array(event.user.to_bytes()),
                nft_mint: Pubkey::new_from_array(event.nft_mint.to_bytes()),
                level: event.level,
                serial_number: event.serial_number,
                timestamp: event.timestamp,
            })
        } else {
            None
        }
    } else if discriminator == &FORTUNE_DRAWN_DISCRIMINATOR {
        if let Ok(event) = FortuneDrawn::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::FortuneDrawn {
                user: Pubkey::new_from_array(event.user.to_bytes()),
                fortune_result: event.fortune_result,
                used_merit: event.used_merit,
                amulet_dropped: event.amulet_dropped,
                timestamp: event.timestamp,
            })
        } else {
            None
        }
    } else if discriminator == &REWARDS_PROCESSED_DISCRIMINATOR {
        if let Ok(event) = RewardsProcessed::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::RewardsProcessed {
                user: Pubkey::new_from_array(event.user.to_bytes()),
                merit_reward: event.merit_reward,
                incense_points_reward: event.incense_points_reward,
                timestamp: event.timestamp,
            })
        } else {
            None
        }
    } else if discriminator == &WISH_CREATED_DISCRIMINATOR {
        if let Ok(event) = WishCreated::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::WishCreated {
                user: Pubkey::new_from_array(event.user.to_bytes()),
                wish_id: event.wish_id,
                is_anonymous: event.is_anonymous,
                amulet_dropped: event.amulet_dropped,
                timestamp: event.timestamp,
            })
        } else {
            None
        }
    } else {
        // Note: IncenseBurned event not found in IDL, will be handled separately if needed
        None // Unknown event type, silently ignore
    }
}
