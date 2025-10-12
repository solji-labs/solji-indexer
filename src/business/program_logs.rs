use anyhow::Result;
use sqlx::{MySql, Pool, Type};
use std::time::Duration;
use tokio::time::sleep;
use tokio_stream::StreamExt;

use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};

use crate::domain::{
    ActivityEnum, CoinFlip, Destroy, Donate, DonateCountCreated, DrawLots, IncenseBought,
    IncenseBurned, LikeCreated, MedalMinted, MedalUpgraded, SbtMinted, TempleWithdrawal,
    UserActivity, WishCreated,
};
use crate::infra::{
    write_coin_flip_to_db, write_donate_count_created_to_db, write_donate_to_db,
    write_draw_lots_to_db, write_incense_bought_to_db, write_incense_burned_to_db,
    write_like_created_to_db, write_medal_minted_to_db, write_medal_upgraded_to_db,
    write_nft_destroy_to_db, write_sbt_minted_to_db, write_temple_withdrawal_to_db,
    write_user_activity_to_db, write_wish_created_to_db,
};
use crate::utils::{decode_anchor_event, event_disc};

async fn backoff(attempt: u32) {
    let secs = 2u64.saturating_pow(attempt.min(5)).max(1).min(60);
    sleep(Duration::from_secs(secs)).await;
}

pub async fn run_logs(pool: &Pool<MySql>) -> Result<()> {
    let ws_url = std::env::var("WS_URL")?;
    let program: Pubkey = std::env::var("PROGRAM_ID")?.parse()?;

    // event discriminant
    let disc_incense_buy = event_disc("IncenseBoughtEvent");
    let disc_incense_burn = event_disc("IncenseBurnedEvent");
    let disc_donate = event_disc("DonateEvent");
    let disc_medal_minted = event_disc("MedalMintedEvent");
    let disc_medal_upgraded = event_disc("MedalUpgradedEvent");
    let disc_draw_lots = event_disc("DrawLotsEvent");
    let disc_coin_flip = event_disc("CoinFlipEvent");
    let disc_destroy = event_disc("DestroyEvent");
    let disc_donate_count_created = event_disc("DonateCountCreatedEvent");
    let disc_temple_with_drawal = event_disc("TempleWithdrawalEvent");
    let disc_like_created = event_disc("LikeCreatedEvent");
    let disc_wish_created = event_disc("WishCreatedEvent");
    let disc_sbt_minted = event_disc("SbtMintedEvent");
    let disc_user_activity = event_disc("UserActivityEvent");

    let mut attempt: u32 = 0;

    loop {
        println!("[logs] connecting to {}", ws_url);

        let client = match PubsubClient::new(&ws_url).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[logs] connect error: {e:?}");
                attempt += 1;
                backoff(attempt).await;
                continue;
            }
        };

        // 订阅
        let (mut stream, _unsub) = match client
            .logs_subscribe(
                RpcTransactionLogsFilter::Mentions(vec![program.to_string()]),
                RpcTransactionLogsConfig {
                    commitment: Some(CommitmentConfig::confirmed()),
                },
            )
            .await
        {
            Ok(v) => {
                attempt = 0;
                v
            }
            Err(e) => {
                eprintln!("[logs] subscribe error: {e:?}");
                attempt += 1;
                backoff(attempt).await;
                continue;
            }
        };

        while let Some(update) = stream.next().await {
            let logs = &update.value.logs;
            if let Some(evt) = decode_anchor_event::<IncenseBought>(logs, &disc_incense_buy) {
                println!("[logs] IncenseBought");
                write_incense_bought_to_db(pool, evt).await?;
            }
            if let Some(evt) = decode_anchor_event::<IncenseBurned>(logs, &disc_incense_burn) {
                println!("[logs] IncenseBurned");
                write_incense_burned_to_db(pool, evt).await?;
            }
            if let Some(evt) = decode_anchor_event::<Donate>(logs, &disc_donate) {
                println!("[logs] Donate");
                write_donate_to_db(pool, evt).await?;
            }
            if let Some(evt) = decode_anchor_event::<MedalMinted>(logs, &disc_medal_minted) {
                println!("[logs] MedalMinted");
                write_medal_minted_to_db(pool, evt).await?;
            }
            if let Some(evt) = decode_anchor_event::<MedalUpgraded>(logs, &disc_medal_upgraded) {
                println!("[logs] MedalUpgraded");
                write_medal_upgraded_to_db(pool, evt).await?;
            }
            if let Some(evt) = decode_anchor_event::<DrawLots>(logs, &disc_draw_lots) {
                println!("[logs] DrawLots");
                write_draw_lots_to_db(pool, evt).await?;
            }
            if let Some(evt) = decode_anchor_event::<CoinFlip>(logs, &disc_coin_flip) {
                println!("[logs] CoinFlip");
                write_coin_flip_to_db(pool, evt).await?;
            }
            if let Some(evt) = decode_anchor_event::<Destroy>(logs, &disc_destroy) {
                println!("[logs] Destroy");
                write_nft_destroy_to_db(pool, evt).await?;
            }
            if let Some(evt) =
                decode_anchor_event::<DonateCountCreated>(logs, &disc_donate_count_created)
            {
                println!("[logs] DonateCountCreated");
                write_donate_count_created_to_db(pool, evt).await?;
            }
            if let Some(evt) =
                decode_anchor_event::<TempleWithdrawal>(logs, &disc_temple_with_drawal)
            {
                println!("[logs] TempleWithdrawal");
                write_temple_withdrawal_to_db(pool, evt).await?;
            }
            if let Some(evt) = decode_anchor_event::<LikeCreated>(logs, &disc_like_created) {
                println!("[logs] LikeCreated");
                write_like_created_to_db(pool, evt).await?;
            }
            if let Some(evt) = decode_anchor_event::<WishCreated>(logs, &disc_wish_created) {
                println!("[logs] WishCreated");
                write_wish_created_to_db(pool, evt).await?;
            }
            if let Some(evt) = decode_anchor_event::<SbtMinted>(logs, &disc_sbt_minted) {
                println!("[logs] SbtMinted");
                write_sbt_minted_to_db(pool, evt).await?;
            }
            if let Some(evt) = decode_anchor_event::<UserActivity>(logs, &disc_user_activity) {
                println!("[logs] UserActivity");
                let activity_type_str = get_activity_type_to_string(&evt.activity_type).to_string();
                write_user_activity_to_db(pool, evt, activity_type_str).await?;
            }
        }

        eprintln!("[logs] stream ended; reconnecting...");
        attempt += 1;
        backoff(attempt).await;
    }
}

fn get_activity_type_to_string(at: &ActivityEnum) -> &str {
    match at {
        ActivityEnum::Burn => "Burn",
        ActivityEnum::Donate => "Donate",
        ActivityEnum::Lottery => "Lottery",
        ActivityEnum::Wish => "Wish",
        ActivityEnum::Like => "Like",
    }
}
