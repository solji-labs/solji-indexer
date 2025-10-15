use anchor_lang::prelude::*;
use base64::{engine::general_purpose, Engine};
use solana_sdk::pubkey::Pubkey;

use crate::events::ProgramEvent;
use temple::state::event::{
    AmuletDropped, AmuletMinted, DonationCompleted, DonationNFTMinted, FortuneDrawn,
    FortuneNFTMinted, IncenseBurned, RewardsProcessed, WishCreated,
};

// Event discriminators from IDL
const DONATION_COMPLETED_DISCRIMINATOR: [u8; 8] = [34, 178, 117, 6, 39, 189, 241, 48];
const DONATION_NFT_MINTED_DISCRIMINATOR: [u8; 8] = [142, 88, 211, 148, 62, 90, 172, 20];
const FORTUNE_DRAWN_DISCRIMINATOR: [u8; 8] = [134, 252, 88, 211, 24, 112, 209, 240];
const FORTUNE_NFT_MINTED_DISCRIMINATOR: [u8; 8] = [226, 138, 253, 243, 89, 224, 0, 199];
const AMULET_DROPPED_DISCRIMINATOR: [u8; 8] = [24, 100, 210, 40, 5, 63, 105, 27];
const AMULET_MINTED_DISCRIMINATOR: [u8; 8] = [5, 74, 5, 29, 227, 131, 7, 204];
const REWARDS_PROCESSED_DISCRIMINATOR: [u8; 8] = [217, 74, 206, 32, 228, 181, 17, 146];
const WISH_CREATED_DISCRIMINATOR: [u8; 8] = [225, 167, 37, 207, 75, 1, 226, 130];
const INCENSE_BURNED_DISCRIMINATOR: [u8; 8] = [211, 166, 224, 11, 104, 105, 175, 186];

const EVENT_PREFIX: &str = "Program log: ";
const ANCHOR_EVENT_LABEL: &str = "Event:";
const PROGRAM_DATA_PREFIX: &str = "Program data: ";

/// Core function: Try to parse an Anchor event from a log string
pub fn try_parse_event(log_str: &str) -> Option<ProgramEvent> {
    println!("try_parse_event called with log: {}", log_str);

    // Check for "Program log: Event:" format first
    if let Some(event_log) = log_str.strip_prefix(EVENT_PREFIX) {
        println!("Found event prefix, remaining: {}", event_log);
        if let Some(base64_data) = event_log.strip_prefix(ANCHOR_EVENT_LABEL) {
            println!("Found anchor event label, base64 data: {}", base64_data);
            return parse_base64_event(base64_data);
        }
    }

    // Also check for "Program data:" format (contains event data)
    if let Some(data_part) = log_str.strip_prefix(PROGRAM_DATA_PREFIX) {
        println!(
            "Found program data prefix, base64 data: {}",
            data_part.trim()
        );
        return parse_base64_event(data_part.trim());
    }

    None
}

fn parse_base64_event(base64_data: &str) -> Option<ProgramEvent> {
    // 2. Base64 decode
    let decoded_bytes = match general_purpose::STANDARD.decode(base64_data.trim()) {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("Base64 decode failed: {:?}", e);
            return None;
        }
    };

    println!("Decoded {} bytes", decoded_bytes.len());

    // Ensure data is long enough (8 bytes discriminator)
    if decoded_bytes.len() < 8 {
        println!("Data too short, need at least 8 bytes");
        return None;
    }

    // Extract discriminator
    let discriminator = &decoded_bytes[0..8];
    println!("Discriminator: {:?}", discriminator);

    // 3. Match discriminator and deserialize (skip the 8-byte discriminator)
    if discriminator == &DONATION_COMPLETED_DISCRIMINATOR {
        println!("Matched DONATION_COMPLETED_DISCRIMINATOR");
        if let Ok(event) = DonationCompleted::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::DonationCompleted {
                user: Pubkey::new_from_array(event.user.to_bytes()),
                amount: event.amount,
                total_donated: event.total_donated,
                level: event.level,
                timestamp: event.timestamp,
            })
        } else {
            println!("Failed to deserialize DonationCompleted");
            None
        }
    } else if discriminator == &DONATION_NFT_MINTED_DISCRIMINATOR {
        println!("Matched DONATION_NFT_MINTED_DISCRIMINATOR");
        if let Ok(event) = DonationNFTMinted::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::DonationNFTMinted {
                user: Pubkey::new_from_array(event.user.to_bytes()),
                nft_mint: Pubkey::new_from_array(event.nft_mint.to_bytes()),
                level: event.level,
                serial_number: event.serial_number,
                timestamp: event.timestamp,
            })
        } else {
            println!("Failed to deserialize DonationNFTMinted");
            None
        }
    } else if discriminator == &FORTUNE_DRAWN_DISCRIMINATOR {
        println!("Matched FORTUNE_DRAWN_DISCRIMINATOR");
        if let Ok(event) = FortuneDrawn::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::FortuneDrawn {
                user: Pubkey::new_from_array(event.user.to_bytes()),
                fortune_result: event.fortune_result,
                used_merit: event.used_merit,
                amulet_dropped: event.amulet_dropped,
                timestamp: event.timestamp,
            })
        } else {
            println!("Failed to deserialize FortuneDrawn");
            None
        }
    } else if discriminator == &REWARDS_PROCESSED_DISCRIMINATOR {
        println!("Matched REWARDS_PROCESSED_DISCRIMINATOR");
        if let Ok(event) = RewardsProcessed::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::RewardsProcessed {
                user: Pubkey::new_from_array(event.user.to_bytes()),
                merit_reward: event.merit_reward,
                incense_points_reward: event.incense_points_reward,
                timestamp: event.timestamp,
            })
        } else {
            println!("Failed to deserialize RewardsProcessed");
            None
        }
    } else if discriminator == &WISH_CREATED_DISCRIMINATOR {
        println!("Matched WISH_CREATED_DISCRIMINATOR");
        println!("Decoded bytes length: {}", decoded_bytes.len());
        println!(
            "Event data (after discriminator): {:?}",
            &decoded_bytes[8..]
        );
        if let Ok(event) = WishCreated::try_from_slice(&decoded_bytes[8..]) {
            println!("Successfully deserialized WishCreated: user={}, wish_id={}, is_anonymous={}, amulet_dropped={}, timestamp={}",
                     event.user, event.wish_id, event.is_anonymous, event.amulet_dropped, event.timestamp);
            Some(ProgramEvent::WishCreated {
                user: Pubkey::new_from_array(event.user.to_bytes()),
                wish_id: event.wish_id,
                content_hash: event.content_hash,
                is_anonymous: event.is_anonymous,
                amulet_dropped: event.amulet_dropped,
                timestamp: event.timestamp,
            })
        } else {
            println!("Failed to deserialize WishCreated");
            None
        }
    } else if discriminator == &AMULET_DROPPED_DISCRIMINATOR {
        println!("Matched AMULET_DROPPED_DISCRIMINATOR");
        if let Ok(event) = AmuletDropped::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::AmuletDropped {
                user: Pubkey::new_from_array(event.user.to_bytes()),
                amulet_type: event.amulet_type,
                source: event.source,
                timestamp: event.timestamp,
            })
        } else {
            println!("Failed to deserialize AmuletDropped");
            None
        }
    } else if discriminator == &AMULET_MINTED_DISCRIMINATOR {
        println!("Matched AMULET_MINTED_DISCRIMINATOR");
        if let Ok(event) = AmuletMinted::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::AmuletMinted {
                user: Pubkey::new_from_array(event.user.to_bytes()),
                amulet_mint: Pubkey::new_from_array(event.amulet_mint.to_bytes()),
                source: event.source,
                serial_number: event.serial_number,
                timestamp: event.timestamp,
            })
        } else {
            println!("Failed to deserialize AmuletMinted");
            None
        }
    } else if discriminator == &FORTUNE_NFT_MINTED_DISCRIMINATOR {
        println!("Matched FORTUNE_NFT_MINTED_DISCRIMINATOR");
        if let Ok(event) = FortuneNFTMinted::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::FortuneNFTMinted {
                user: Pubkey::new_from_array(event.user.to_bytes()),
                fortune_nft_mint: Pubkey::new_from_array(event.fortune_nft_mint.to_bytes()),
                fortune_result: event.fortune_result,
                merit_cost: event.merit_cost,
                serial_number: event.serial_number,
                timestamp: event.timestamp,
            })
        } else {
            println!("Failed to deserialize FortuneNFTMinted");
            None
        }
    } else if discriminator == &INCENSE_BURNED_DISCRIMINATOR {
        println!("Matched INCENSE_BURNED_DISCRIMINATOR");
        if let Ok(event) = IncenseBurned::try_from_slice(&decoded_bytes[8..]) {
            println!(
                "Successfully parsed IncenseBurned event: user={}, incense_id={}, amount={}",
                event.user, event.incense_id, event.amount
            );
            Some(ProgramEvent::IncenseBurned {
                user: Pubkey::new_from_array(event.user.to_bytes()),
                incense_id: event.incense_id,
                amount: event.amount,
                timestamp: event.timestamp,
            })
        } else {
            println!("Failed to deserialize IncenseBurned");
            None
        }
    } else {
        println!("Unknown discriminator: {:?}", discriminator);
        None // Unknown event type, silently ignore
    }
}
