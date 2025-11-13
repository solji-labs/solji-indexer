use base64::{engine::general_purpose, Engine};
use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

use crate::events::{
    discriminators, AmuletDropped, AmuletMinted, DonationCompleted, DonationNFTMinted,
    FortuneDrawn, FortuneNFTMinted, IncenseBurned, ProgramEvent, RewardsProcessed,
    ShopConfigUpdated, WishCreated, WishTowerUpdated,
};

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
    if discriminator == discriminators::DONATION_COMPLETED {
        println!("Matched DONATION_COMPLETED_DISCRIMINATOR");
        if let Ok(event) = DonationCompleted::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::DonationCompleted {
                user: Pubkey::new_from_array(event.user),
                amount: event.amount,
                total_donated: event.total_donated,
                level: event.level,
                timestamp: event.timestamp,
                transaction_signature: String::new(),
            })
        } else {
            println!("Failed to deserialize DonationCompleted");
            None
        }
    } else if discriminator == discriminators::DONATION_NFT_MINTED {
        println!("Matched DONATION_NFT_MINTED_DISCRIMINATOR");
        if let Ok(event) = DonationNFTMinted::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::DonationNFTMinted {
                user: Pubkey::new_from_array(event.user),
                nft_mint: Pubkey::new_from_array(event.nft_mint),
                level: event.level,
                serial_number: event.serial_number,
                timestamp: event.timestamp,
                transaction_signature: String::new(),
            })
        } else {
            println!("Failed to deserialize DonationNFTMinted");
            None
        }
    } else if discriminator == discriminators::FORTUNE_DRAWN {
        println!("Matched FORTUNE_DRAWN_DISCRIMINATOR");
        if let Ok(event) = FortuneDrawn::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::FortuneDrawn {
                user: Pubkey::new_from_array(event.user),
                fortune_result: event.fortune_result,
                used_merit: event.used_merit,
                amulet_dropped: event.amulet_dropped,
                timestamp: event.timestamp,
                transaction_signature: String::new(),
            })
        } else {
            println!("Failed to deserialize FortuneDrawn");
            None
        }
    } else if discriminator == discriminators::REWARDS_PROCESSED {
        println!("Matched REWARDS_PROCESSED_DISCRIMINATOR");
        if let Ok(event) = RewardsProcessed::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::RewardsProcessed {
                user: Pubkey::new_from_array(event.user),
                merit_reward: event.merit_reward,
                incense_points_reward: event.incense_points_reward,
                timestamp: event.timestamp,
                transaction_signature: String::new(),
            })
        } else {
            println!("Failed to deserialize RewardsProcessed");
            None
        }
    } else if discriminator == discriminators::WISH_CREATED {
        println!("Matched WISH_CREATED_DISCRIMINATOR");
        println!("Decoded bytes length: {}", decoded_bytes.len());
        println!(
            "Event data (after discriminator): {:?}",
            &decoded_bytes[8..]
        );
        if let Ok(event) = WishCreated::try_from_slice(&decoded_bytes[8..]) {
            println!("Successfully deserialized WishCreated: user={:?}, wish_id={}, is_anonymous={}, amulet_dropped={}, timestamp={}",
                     event.user, event.wish_id, event.is_anonymous, event.amulet_dropped, event.timestamp);
            Some(ProgramEvent::WishCreated {
                user: Pubkey::new_from_array(event.user),
                wish_id: event.wish_id,
                content_hash: event.content_hash,
                is_anonymous: event.is_anonymous,
                amulet_dropped: event.amulet_dropped,
                timestamp: event.timestamp,
                transaction_signature: String::new(),
            })
        } else {
            println!("Failed to deserialize WishCreated");
            None
        }
    } else if discriminator == discriminators::AMULET_DROPPED {
        println!("Matched AMULET_DROPPED_DISCRIMINATOR");
        if let Ok(event) = AmuletDropped::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::AmuletDropped {
                user: Pubkey::new_from_array(event.user),
                amulet_type: event.amulet_type,
                source: event.source,
                timestamp: event.timestamp,
                transaction_signature: String::new(),
            })
        } else {
            println!("Failed to deserialize AmuletDropped");
            None
        }
    } else if discriminator == discriminators::AMULET_MINTED {
        println!("Matched AMULET_MINTED_DISCRIMINATOR");
        if let Ok(event) = AmuletMinted::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::AmuletMinted {
                user: Pubkey::new_from_array(event.user),
                amulet_mint: Pubkey::new_from_array(event.amulet_mint),
                source: event.source,
                serial_number: event.serial_number,
                timestamp: event.timestamp,
                transaction_signature: String::new(),
            })
        } else {
            println!("Failed to deserialize AmuletMinted");
            None
        }
    } else if discriminator == discriminators::FORTUNE_NFT_MINTED {
        println!("Matched FORTUNE_NFT_MINTED_DISCRIMINATOR");
        if let Ok(event) = FortuneNFTMinted::try_from_slice(&decoded_bytes[8..]) {
            Some(ProgramEvent::FortuneNFTMinted {
                user: Pubkey::new_from_array(event.user),
                fortune_nft_mint: Pubkey::new_from_array(event.fortune_nft_mint),
                fortune_result: event.fortune_result,
                merit_cost: event.merit_cost,
                serial_number: event.serial_number,
                timestamp: event.timestamp,
                transaction_signature: String::new(),
            })
        } else {
            println!("Failed to deserialize FortuneNFTMinted");
            None
        }
    } else if discriminator == discriminators::INCENSE_BURNED {
        println!("Matched INCENSE_BURNED_DISCRIMINATOR");
        if let Ok(event) = IncenseBurned::try_from_slice(&decoded_bytes[8..]) {
            println!(
                "Successfully parsed IncenseBurned event: user={:?}, incense_id={}, amount={}",
                event.user, event.incense_id, event.amount
            );
            Some(ProgramEvent::IncenseBurned {
                user: Pubkey::new_from_array(event.user),
                incense_id: event.incense_id,
                amount: event.amount,
                timestamp: event.timestamp,
                transaction_signature: String::new(),
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
