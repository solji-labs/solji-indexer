use std::time::Duration;

use crate::{
    domain::{
        CoinFlip, CoinFlipResp, Destroy, DestroyResp, Donate, DonateCountCreated,
        DonateCountCreatedResp, DonateResp, DrawLots, DrawLotsResp, IncenseBought,
        IncenseBoughtResp, IncenseBurned, IncenseBurnedResp, LikeCreated, LikeCreatedResp,
        MedalMinted, MedalMintedResp, MedalUpgraded, MedalUpgradedResp, PageReq, SbtMinted,
        SbtMintedResp, Temple, TempleResp, TempleWithdrawal, TempleWithdrawalResp, TimeReq,
        UserActivity, UserActivityResp, UserDonateResp, UserInfo, UserInfoResp, UserMedalResp,
        WishCreated, WishCreatedResp,
    },
    utils::PageResp,
};
use anyhow::{Context, Ok, Result};
use sqlx::{mysql::MySqlPoolOptions, types::Json, MySql, MySqlPool, Pool, QueryBuilder};

pub async fn connect_db() -> Result<MySqlPool> {
    let url = std::env::var("DATABASE_URL").context("DATABASE_URL is not set")?;

    let pool = MySqlPoolOptions::new()
        .max_connections(20) // 根据并发调整，10~50 之间看业务
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(10)) // 等待空闲连接的超时
        .idle_timeout(Duration::from_secs(60))
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET time_zone = '+08:00'")
                    .execute(conn)
                    .await
                    .map(|_| ()) // <-- Result<(), sqlx::Error>
            })
        })
        .connect(&url)
        .await?;

    Ok(pool)
}

pub async fn write_incense_bought_to_db(pool: &Pool<MySql>, evt: IncenseBought) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO incense_bought
          (user, incense_type, number, unit_price, total_amount, event_time,timestamp)
        VALUES (?, ?, ?, ?, ?, FROM_UNIXTIME(?), ?)
        "#,
    )
    .bind(evt.buyer.to_string())
    .bind(evt.incense_type)
    .bind(evt.number)
    .bind(evt.unit_price)
    .bind(evt.total_amount)
    .bind(evt.timestamp)
    .bind(evt.timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn write_incense_burned_to_db(pool: &Pool<MySql>, evt: IncenseBurned) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO incense_burned
          (user, incense_type, nft_mint, incense_value, merit_value, event_time, timestamp)
        VALUES (?, ?, ?, ?, ?, FROM_UNIXTIME(?), ?)
        "#,
    )
    .bind(evt.user.to_string())
    .bind(evt.incense_type) // String
    .bind(evt.nft_mint.to_string())
    .bind(evt.incense_value) // u64 -> BIGINT UNSIGNED
    .bind(evt.merit_value) // u64 -> BIGINT UNSIGNED
    .bind(evt.timestamp) // i64  -> event_time
    .bind(evt.timestamp) // i64 -> timestamp
    .execute(pool)
    .await?;
    Ok(())
}

// 捐赠
pub async fn write_donate_to_db(pool: &Pool<MySql>, evt: Donate) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO donate
          (user, amount, merit_value, incense_value, event_time, timestamp)
        VALUES (?, ?, ?, ?, FROM_UNIXTIME(?), ?)
        "#,
    )
    .bind(evt.user.to_string())
    .bind(evt.amount) // u64
    .bind(evt.merit_value) // u64
    .bind(evt.incense_value) // u64
    .bind(evt.timestamp) // -> event_time
    .bind(evt.timestamp) // -> timestamp
    .execute(pool)
    .await?;
    Ok(())
}

// 奖章铸造
pub async fn write_medal_minted_to_db(pool: &Pool<MySql>, evt: MedalMinted) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO medal_minted
          (user, level, nft_mint, event_time, timestamp)
        VALUES (?, ?, ?, FROM_UNIXTIME(?), ?)
        "#,
    )
    .bind(evt.user.to_string())
    .bind(evt.level) // String
    .bind(evt.nft_mint.to_string())
    .bind(evt.timestamp)
    .bind(evt.timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

// 奖章升级
pub async fn write_medal_upgraded_to_db(pool: &Pool<MySql>, evt: MedalUpgraded) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO medal_upgraded
          (user, old_level, new_level, nft_mint, event_time, timestamp)
        VALUES (?, ?, ?, ?, FROM_UNIXTIME(?), ?)
        "#,
    )
    .bind(evt.user.to_string())
    .bind(evt.old_level) // String
    .bind(evt.new_level) // String
    .bind(evt.nft_mint.to_string())
    .bind(evt.timestamp)
    .bind(evt.timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

// 抽签
pub async fn write_draw_lots_to_db(pool: &Pool<MySql>, evt: DrawLots) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO draw_lots
          (user, lottery_type,lottery_poetry, merit_change, event_time, timestamp)
        VALUES (?, ?, ?,?, FROM_UNIXTIME(?), ?)
        "#,
    )
    .bind(evt.user.to_string())
    .bind(evt.lottery_type) // enum -> String
    .bind(evt.lottery_poetry) // String
    .bind(evt.merit_change) // u64
    .bind(evt.timestamp)
    .bind(evt.timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

// 抛硬币
pub async fn write_coin_flip_to_db(pool: &Pool<MySql>, evt: CoinFlip) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO coin_flip
          (player, randomness_account, commit_slot, event_time, timestamp)
        VALUES (?, ?, ?, FROM_UNIXTIME(?), ?)
        "#,
    )
    .bind(evt.player.to_string())
    .bind(evt.randomness_account.to_string())
    .bind(evt.commit_slot) // u64 -> BIGINT UNSIGNED
    .bind(evt.timestamp)
    .bind(evt.timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

// 销毁NFT
pub async fn write_nft_destroy_to_db(pool: &Pool<MySql>, evt: Destroy) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO nft_destroy
          (user, mint, event_time, timestamp)
        VALUES (?, ?, FROM_UNIXTIME(?), ?)
        "#,
    )
    .bind(evt.user.to_string())
    .bind(evt.mint.to_string())
    .bind(evt.timestamp)
    .bind(evt.timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

// 捐赠计数创建
pub async fn write_donate_count_created_to_db(
    pool: &Pool<MySql>,
    evt: DonateCountCreated,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO donate_count_created
          (authority, event_time, timestamp)
        VALUES (?, FROM_UNIXTIME(?), ?)
        "#,
    )
    .bind(evt.authority.to_string())
    .bind(evt.timestamp)
    .bind(evt.timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn write_temple_withdrawal_to_db(
    pool: &Pool<MySql>,
    evt: TempleWithdrawal,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO temple_withdrawal
            (admin, amount, remaining_balance, event_time, `timestamp`)
        VALUES
            (?, ?, ?, FROM_UNIXTIME(?), ?)
        "#,
    )
    .bind(evt.admin.to_string())
    .bind(evt.amount)
    .bind(evt.remaining_balance)
    .bind(evt.timestamp)
    .bind(evt.timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn write_like_created_to_db(pool: &Pool<MySql>, evt: LikeCreated) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO like_created
            (`user`, `wish`, new_like_count, event_time, `timestamp`)
        VALUES
            (?, ?, ?, FROM_UNIXTIME(?), ?)
        "#,
    )
    .bind(evt.user.to_string())
    .bind(evt.wish.to_string())
    .bind(evt.new_like_count)
    .bind(evt.timestamp)
    .bind(evt.timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn write_wish_created_to_db(pool: &Pool<MySql>, evt: WishCreated) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO wish_created
            (`user`, content, `value`, is_anonymous, event_time, `timestamp`)
        VALUES
            (?, ?, ?, ?, FROM_UNIXTIME(?), ?)
        "#,
    )
    .bind(evt.user.to_string())
    .bind(evt.content)
    .bind(evt.value as i32)
    .bind(evt.is_anonymous)
    .bind(evt.timestamp)
    .bind(evt.timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn write_sbt_minted_to_db(pool: &Pool<MySql>, evt: SbtMinted) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO sbt_minted
            (`user`, mint, ata, `name`, symbol, url, donate_amount, event_time, `timestamp`)
        VALUES
            (?, ?, ?, ?, ?, ?, ?, FROM_UNIXTIME(?), ?)
        "#,
    )
    .bind(evt.user.to_string())
    .bind(evt.mint.to_string())
    .bind(evt.ata.to_string())
    .bind(evt.name)
    .bind(evt.symbol)
    .bind(evt.url)
    .bind(evt.donate_amount)
    .bind(evt.timestamp)
    .bind(evt.timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn write_user_activity_to_db(
    pool: &Pool<MySql>,
    evt: UserActivity,
    activity_type: String,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO user_activity
            (`user`, activity_type, content, event_time,`timestamp`)
        VALUES
            (?, ?, ?, FROM_UNIXTIME(?),?)
        "#,
    )
    .bind(evt.user.to_string())
    .bind(activity_type)
    .bind(evt.content)
    .bind(evt.timestamp)
    .bind(evt.timestamp)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn write_user_info_to_db(
    pool: &Pool<MySql>,
    evt: UserInfo,
    medel_level: u8,
) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO user_info (
            pubkey,
            burn_count,
            total_burn_count,
            incense_buy_count,
            incense_donate_count,
            incense_value,
            merit_value,
            incense_time,
            donate_amount,
            donate_count,
            donate_merit_value,
            donate_incense_value,
            current_medal_level,
            lottery_count,
            lottery_is_free,
            lottery_time,
            wish_count,
            wish_update_time,
            wish_daily_count,
            amulet_count,
            has_sbt_token,
            has_burn_token,
            create_time,
            update_time
        )
        VALUES (
            ?, ?, ?, ?, ?,?, ?, FROM_UNIXTIME(?),
            ?,?, ?, ?, ?, ?, ?, FROM_UNIXTIME(?),
            ?, FROM_UNIXTIME(?), ?,?, ?,?, NOW(), NOW()
        )
        ON DUPLICATE KEY UPDATE
            burn_count               = ?,
            total_burn_count         = ?,
            incense_buy_count        = ?,
            incense_donate_count     = ?,
            incense_value            = ?,
            merit_value              = ?,
            incense_time             = FROM_UNIXTIME(?),
            donate_amount            = ?,
            donate_count             = ?,
            donate_merit_value       = ?,
            donate_incense_value     = ?,
            current_medal_level      = ?,
            lottery_count            = ?,
            lottery_is_free          = ?,
            lottery_time             = FROM_UNIXTIME(?),
            wish_count               = ?,
            wish_update_time         = FROM_UNIXTIME(?),
            wish_daily_count         = ?,
            amulet_count             = ?,
            has_sbt_token            = ?,
            has_burn_token           = ?,
            update_time              = NOW()
        "#,
    )
    // INSERT 部分
    .bind(evt.user.to_string())
    .bind(Json(evt.burn_count))
    .bind(evt.total_burn_count)
    .bind(Json(evt.incense_buy_count))
    .bind(Json(evt.incense_donate_count))
    .bind(evt.incense_value)
    .bind(evt.merit_value)
    .bind(evt.incense_time)
    .bind(evt.donate_amount)
    .bind(evt.donate_count)
    .bind(evt.donate_merit_value)
    .bind(evt.donate_incense_value)
    .bind(medel_level)
    .bind(evt.lottery_count)
    .bind(evt.lottery_is_free)
    .bind(evt.lottery_time)
    .bind(evt.wish_count)
    .bind(evt.wish_update_time)
    .bind(evt.wish_daily_count)
    .bind(evt.amulet_count)
    .bind(evt.has_sbt_token)
    .bind(Json(evt.has_burn_token))
    // UPDATE 部分（再绑定一遍）
    .bind(Json(evt.burn_count))
    .bind(evt.total_burn_count)
    .bind(Json(evt.incense_buy_count))
    .bind(Json(evt.incense_donate_count))
    .bind(evt.incense_value)
    .bind(evt.merit_value)
    .bind(evt.incense_time)
    .bind(evt.donate_amount)
    .bind(evt.donate_count)
    .bind(evt.donate_merit_value)
    .bind(evt.donate_incense_value)
    .bind(medel_level)
    .bind(evt.lottery_count)
    .bind(evt.lottery_is_free)
    .bind(evt.lottery_time)
    .bind(evt.wish_count)
    .bind(evt.wish_update_time)
    .bind(evt.wish_daily_count)
    .bind(evt.amulet_count)
    .bind(evt.has_sbt_token)
    .bind(Json(evt.has_burn_token))
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn write_temple_to_db(pool: &Pool<MySql>, evt: Temple) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO temple (
            admin,
            level,
            total_incense_value,
            total_merit_value,
            total_burn_count,
            total_lottery_count,
            total_wish_count,
            total_donate_amount,
            total_donate_count,
            total_amulet_count,
            buddha_nft_count,
            wealth,
            create_time,
            update_time
        )
        VALUES (
            ?, ?,?, ?,?, ?,?, ?,?,?,?,?, NOW(), NOW()
        )
        ON DUPLICATE KEY UPDATE
            admin                 = ?,
            level                 = ?,
            total_incense_value   = ?,
            total_merit_value     = ?,
            total_burn_count     = ?,
            total_lottery_count   = ?,
            total_wish_count      = ?,
            total_donate_amount   = ?,
            total_donate_count   = ?,
            total_amulet_count    = ?,
            buddha_nft_count      = ?,
            wealth                = ?,
            update_time           = NOW()
        "#,
    )
    // INSERT 部分
    .bind(evt.admin.to_string())
    .bind(evt.level)
    .bind(evt.total_incense_value)
    .bind(evt.total_merit_value)
    .bind(evt.total_burn_count)
    .bind(evt.total_lottery_count)
    .bind(evt.total_wish_count)
    .bind(evt.total_donate_amount)
    .bind(evt.total_donate_count)
    .bind(evt.total_amulet_count)
    .bind(evt.buddha_nft_count)
    .bind(evt.wealth)
    // UPDATE 部分（再绑定一遍）
    .bind(evt.admin.to_string())
    .bind(evt.level)
    .bind(evt.total_incense_value)
    .bind(evt.total_merit_value)
    .bind(evt.total_burn_count)
    .bind(evt.total_lottery_count)
    .bind(evt.total_wish_count)
    .bind(evt.total_donate_amount)
    .bind(evt.total_donate_count)
    .bind(evt.total_amulet_count)
    .bind(evt.buddha_nft_count)
    .bind(evt.wealth)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_temple_by_admin(
    pool: &Pool<MySql>,
    admin_pubkey: &str,
) -> Result<Option<TempleResp>, sqlx::Error> {
    sqlx::query_as::<_, TempleResp>(
        r#"
        SELECT
            *
        FROM temple
        WHERE admin = ? and is_deleted = 0
        LIMIT 1
        "#,
    )
    .bind(admin_pubkey)
    .fetch_optional(pool)
    .await
}

pub async fn get_user_info_by_pubkey(
    pool: &Pool<MySql>,
    pubkey: &str,
) -> Result<Option<UserInfoResp>, sqlx::Error> {
    sqlx::query_as::<_, UserInfoResp>(
        r#"
        SELECT
         *
        FROM user_info
        WHERE pubkey = ? AND is_deleted = 0
        LIMIT 1
        "#,
    )
    .bind(pubkey)
    .fetch_optional(pool)
    .await
}

pub async fn page_user_info(pool: &Pool<MySql>, q: &PageReq) -> Result<PageResp<UserInfoResp>> {
    let page_num = q.page_num.max(1);
    let page_size = q.page_size.clamp(1, 50);
    let offset = (page_num - 1) as u64 * page_size as u64;

    // 1) 统计总数
    let mut count_sql =
        QueryBuilder::<MySql>::new("SELECT COUNT(*) AS cnt FROM user_info WHERE is_deleted = 0");
    if let Some(pk) = &q.pubkey {
        count_sql.push(" AND pubkey = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        count_sql.push(" AND update_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        count_sql.push(" AND update_time <= ").push_bind(et);
    }
    let total: i64 = count_sql.build_query_scalar().fetch_one(pool).await?;

    let mut list_sql = QueryBuilder::<MySql>::new(
        r#"
        SELECT
         *
        FROM user_info
        WHERE is_deleted = 0
        "#,
    );
    if let Some(pk) = &q.pubkey {
        list_sql.push(" AND pubkey = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        list_sql.push(" AND update_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND update_time <= ").push_bind(et);
    }
    list_sql.push(" ORDER BY update_time DESC, id DESC ");
    list_sql.push(" LIMIT ").push_bind(page_size as i64);
    list_sql.push(" OFFSET ").push_bind(offset as i64);

    let list: Vec<UserInfoResp> = list_sql.build_query_as().fetch_all(pool).await?;

    Ok(PageResp {
        list,
        total: total as u64,
        page_num,
        page_size,
    })
}

pub async fn query_user_count(pool: &Pool<MySql>, q: &TimeReq) -> Result<i64, sqlx::Error> {
    let mut count_sql = sqlx::QueryBuilder::<MySql>::new(
        "SELECT COUNT(*) AS cnt FROM user_info WHERE is_deleted = 0",
    );
    if let Some(st) = &q.start_time {
        count_sql.push(" AND update_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        count_sql.push(" AND update_time <= ").push_bind(et);
    }
    count_sql.build_query_scalar().fetch_one(pool).await
}

pub async fn query_user_donation_ranking(
    pool: &Pool<MySql>,
) -> Result<Vec<UserDonateResp>, sqlx::Error> {
    sqlx::query_as::<_, UserDonateResp>("SELECT pubkey,current_medal_level,donate_amount FROM user_info order by donate_amount DESC limit 5")
        .fetch_all(pool)
        .await
}

pub async fn query_user_merit_ranking(
    pool: &Pool<MySql>,
    q: &TimeReq,
) -> Result<Vec<UserMedalResp>, sqlx::Error> {
    let mut list_sql = sqlx::QueryBuilder::<MySql>::new(
        "SELECT pubkey,current_medal_level, (merit_value + donate_merit_value) AS total_merit FROM user_info ORDER BY 
  (merit_value + donate_merit_value) DESC limit 5",
    );
    if let Some(st) = &q.start_time {
        list_sql.push(" AND update_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND update_time <= ").push_bind(et);
    }
    list_sql.build_query_as().fetch_all(pool).await
}
pub async fn query_user_activity(
    pool: &Pool<MySql>,
    q: &TimeReq,
) -> Result<Vec<UserActivityResp>, sqlx::Error> {
    let mut list_sql = sqlx::QueryBuilder::<MySql>::new(
        "SELECT * from user_activity ORDER BY event_time DESC limit 5",
    );
    if let Some(st) = &q.start_time {
        list_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND event_time <= ").push_bind(et);
    }
    list_sql.build_query_as().fetch_all(pool).await
}

pub async fn query_interactions_count(
    pool: &Pool<MySql>,
    pubkey: &str,
) -> anyhow::Result<Option<u64>> {
    let total = sqlx::query_scalar::<_, u64>(
        r#"
        SELECT CAST(
            (total_burn_count
           + total_lottery_count
           + total_wish_count
           + total_donate_count
           ) AS UNSIGNED
        )
        FROM temple
        WHERE admin = ? AND is_deleted = 0
        LIMIT 1
        "#,
    )
    .bind(pubkey)
    .fetch_optional(pool)
    .await?;
    Ok(total)
}

pub async fn page_incense_bought(
    pool: &Pool<MySql>,
    q: &PageReq,
) -> Result<PageResp<IncenseBoughtResp>> {
    let page_num = q.page_num.max(1);
    let page_size = q.page_size.clamp(1, 50);
    let offset = (page_num - 1) as u64 * page_size as u64;
    let mut count_sql = QueryBuilder::<MySql>::new(
        "SELECT COUNT(*) as cnt FROM incense_bought WHERE is_deleted = 0",
    );
    if let Some(pk) = &q.pubkey {
        count_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        count_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        count_sql.push(" AND event_time <= ").push_bind(et);
    }
    let total: i64 = count_sql.build_query_scalar().fetch_one(pool).await?;
    let mut list_sql = QueryBuilder::<MySql>::new(
        r#"
        SELECT
        *
        FROM incense_bought
        WHERE is_deleted = 0
        "#,
    );
    if let Some(pk) = &q.pubkey {
        list_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        list_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND event_time <= ").push_bind(et);
    }
    list_sql.push(" ORDER BY event_time DESC, id DESC ");
    list_sql.push(" LIMIT ").push_bind(page_size as i64);
    list_sql.push(" OFFSET ").push_bind(offset as i64);
    let list: Vec<IncenseBoughtResp> = list_sql.build_query_as().fetch_all(pool).await?;
    Ok(PageResp {
        list,
        total: total as u64,
        page_num,
        page_size,
    })
}

pub async fn page_incense_burned(
    pool: &Pool<MySql>,
    q: &PageReq,
) -> Result<PageResp<IncenseBurnedResp>> {
    let page_num = q.page_num.max(1);
    let page_size = q.page_size.clamp(1, 50);
    let offset = (page_num - 1) as u64 * page_size as u64;
    let mut count_sql = QueryBuilder::<MySql>::new(
        "SELECT COUNT(*) as cnt FROM incense_burned WHERE is_deleted = 0",
    );
    if let Some(pk) = &q.pubkey {
        count_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        count_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        count_sql.push(" AND event_time <= ").push_bind(et);
    }
    let total: i64 = count_sql.build_query_scalar().fetch_one(pool).await?;
    let mut list_sql = QueryBuilder::<MySql>::new(
        r#"
        SELECT
        *
        FROM incense_burned
        WHERE is_deleted = 0
        "#,
    );
    if let Some(pk) = &q.pubkey {
        list_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        list_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND event_time <= ").push_bind(et);
    }
    list_sql.push(" ORDER BY event_time DESC, id DESC ");
    list_sql.push(" LIMIT ").push_bind(page_size as i64);
    list_sql.push(" OFFSET ").push_bind(offset as i64);
    let list: Vec<IncenseBurnedResp> = list_sql.build_query_as().fetch_all(pool).await?;
    Ok(PageResp {
        list,
        total: total as u64,
        page_num,
        page_size,
    })
}

pub async fn page_donate(pool: &Pool<MySql>, q: &PageReq) -> Result<PageResp<DonateResp>> {
    let page_num = q.page_num.max(1);
    let page_size = q.page_size.clamp(1, 50);
    let offset = (page_num - 1) as u64 * page_size as u64;
    let mut count_sql =
        QueryBuilder::<MySql>::new("SELECT COUNT(*) as cnt FROM donate WHERE is_deleted = 0");
    if let Some(pk) = &q.pubkey {
        count_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        count_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        count_sql.push(" AND event_time <= ").push_bind(et);
    }
    let total: i64 = count_sql.build_query_scalar().fetch_one(pool).await?;
    let mut list_sql = QueryBuilder::<MySql>::new(
        r#"
        SELECT
        *
        FROM donate
        WHERE is_deleted = 0
        "#,
    );
    if let Some(pk) = &q.pubkey {
        list_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        list_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND event_time <= ").push_bind(et);
    }
    list_sql.push(" ORDER BY event_time DESC, id DESC ");
    list_sql.push(" LIMIT ").push_bind(page_size as i64);
    list_sql.push(" OFFSET ").push_bind(offset as i64);
    let list: Vec<DonateResp> = list_sql.build_query_as().fetch_all(pool).await?;
    Ok(PageResp {
        list,
        total: total as u64,
        page_num,
        page_size,
    })
}

pub async fn page_medal_minted(
    pool: &Pool<MySql>,
    q: &PageReq,
) -> Result<PageResp<MedalMintedResp>> {
    let page_num = q.page_num.max(1);
    let page_size = q.page_size.clamp(1, 50);
    let offset = (page_num - 1) as u64 * page_size as u64;
    let mut count_sql =
        QueryBuilder::<MySql>::new("SELECT COUNT(*) as cnt FROM medal_minted WHERE is_deleted = 0");
    if let Some(pk) = &q.pubkey {
        count_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        count_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        count_sql.push(" AND event_time <= ").push_bind(et);
    }
    let total: i64 = count_sql.build_query_scalar().fetch_one(pool).await?;
    let mut list_sql = QueryBuilder::<MySql>::new(
        r#"
        SELECT
        *
        FROM medal_minted
        WHERE is_deleted = 0
        "#,
    );
    if let Some(pk) = &q.pubkey {
        list_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        list_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND event_time <= ").push_bind(et);
    }
    list_sql.push(" ORDER BY event_time DESC, id DESC ");
    list_sql.push(" LIMIT ").push_bind(page_size as i64);
    list_sql.push(" OFFSET ").push_bind(offset as i64);
    let list: Vec<MedalMintedResp> = list_sql.build_query_as().fetch_all(pool).await?;
    Ok(PageResp {
        list,
        total: total as u64,
        page_num,
        page_size,
    })
}

pub async fn page_medal_upgraded(
    pool: &Pool<MySql>,
    q: &PageReq,
) -> Result<PageResp<MedalUpgradedResp>> {
    let page_num = q.page_num.max(1);
    let page_size = q.page_size.clamp(1, 50);
    let offset = (page_num - 1) as u64 * page_size as u64;
    let mut count_sql = QueryBuilder::<MySql>::new(
        "SELECT COUNT(*) as cnt FROM medal_upgraded WHERE is_deleted = 0",
    );
    if let Some(pk) = &q.pubkey {
        count_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        count_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        count_sql.push(" AND event_time <= ").push_bind(et);
    }
    let total: i64 = count_sql.build_query_scalar().fetch_one(pool).await?;
    let mut list_sql = QueryBuilder::<MySql>::new(
        r#"
        SELECT
        *
        FROM medal_upgraded
        WHERE is_deleted = 0
        "#,
    );
    if let Some(pk) = &q.pubkey {
        list_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        list_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND event_time <= ").push_bind(et);
    }
    list_sql.push(" ORDER BY event_time DESC, id DESC ");
    list_sql.push(" LIMIT ").push_bind(page_size as i64);
    list_sql.push(" OFFSET ").push_bind(offset as i64);
    let list: Vec<MedalUpgradedResp> = list_sql.build_query_as().fetch_all(pool).await?;
    Ok(PageResp {
        list,
        total: total as u64,
        page_num,
        page_size,
    })
}

pub async fn page_draw_lots(pool: &Pool<MySql>, q: &PageReq) -> Result<PageResp<DrawLotsResp>> {
    let page_num = q.page_num.max(1);
    let page_size = q.page_size.clamp(1, 50);
    let offset = (page_num - 1) as u64 * page_size as u64;
    let mut count_sql =
        QueryBuilder::<MySql>::new("SELECT COUNT(*) as cnt FROM draw_lots WHERE is_deleted = 0");
    if let Some(pk) = &q.pubkey {
        count_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        count_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        count_sql.push(" AND event_time <= ").push_bind(et);
    }
    let total: i64 = count_sql.build_query_scalar().fetch_one(pool).await?;
    let mut list_sql = QueryBuilder::<MySql>::new(
        r#"
        SELECT
        *
        FROM draw_lots
        WHERE is_deleted = 0
        "#,
    );
    if let Some(pk) = &q.pubkey {
        list_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        list_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND event_time <= ").push_bind(et);
    }
    list_sql.push(" ORDER BY event_time DESC, id DESC ");
    list_sql.push(" LIMIT ").push_bind(page_size as i64);
    list_sql.push(" OFFSET ").push_bind(offset as i64);
    let list: Vec<DrawLotsResp> = list_sql.build_query_as().fetch_all(pool).await?;
    Ok(PageResp {
        list,
        total: total as u64,
        page_num,
        page_size,
    })
}
// =============================================================================================
pub async fn page_coin_flip(pool: &Pool<MySql>, q: &PageReq) -> Result<PageResp<CoinFlipResp>> {
    let page_num = q.page_num.max(1);
    let page_size = q.page_size.clamp(1, 50);
    let offset = (page_num - 1) as u64 * page_size as u64;
    let mut count_sql =
        QueryBuilder::<MySql>::new("SELECT COUNT(*) as cnt FROM coin_flip WHERE is_deleted = 0");
    if let Some(pk) = &q.pubkey {
        count_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        count_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        count_sql.push(" AND event_time <= ").push_bind(et);
    }
    let total: i64 = count_sql.build_query_scalar().fetch_one(pool).await?;
    let mut list_sql = QueryBuilder::<MySql>::new(
        r#"
        SELECT
        *
        FROM coin_flip
        WHERE is_deleted = 0
        "#,
    );
    if let Some(pk) = &q.pubkey {
        list_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        list_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND event_time <= ").push_bind(et);
    }
    list_sql.push(" ORDER BY event_time DESC, id DESC ");
    list_sql.push(" LIMIT ").push_bind(page_size as i64);
    list_sql.push(" OFFSET ").push_bind(offset as i64);
    let list: Vec<CoinFlipResp> = list_sql.build_query_as().fetch_all(pool).await?;
    Ok(PageResp {
        list,
        total: total as u64,
        page_num,
        page_size,
    })
}
pub async fn page_nft_destroy(pool: &Pool<MySql>, q: &PageReq) -> Result<PageResp<DestroyResp>> {
    let page_num = q.page_num.max(1);
    let page_size = q.page_size.clamp(1, 50);
    let offset = (page_num - 1) as u64 * page_size as u64;
    let mut count_sql =
        QueryBuilder::<MySql>::new("SELECT COUNT(*) as cnt FROM nft_destroy WHERE is_deleted = 0");
    if let Some(pk) = &q.pubkey {
        count_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        count_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        count_sql.push(" AND event_time <= ").push_bind(et);
    }
    let total: i64 = count_sql.build_query_scalar().fetch_one(pool).await?;
    let mut list_sql = QueryBuilder::<MySql>::new(
        r#"
        SELECT
        *
        FROM nft_destroy
        WHERE is_deleted = 0
        "#,
    );
    if let Some(pk) = &q.pubkey {
        list_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        list_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND event_time <= ").push_bind(et);
    }
    list_sql.push(" ORDER BY event_time DESC, id DESC ");
    list_sql.push(" LIMIT ").push_bind(page_size as i64);
    list_sql.push(" OFFSET ").push_bind(offset as i64);
    let list: Vec<DestroyResp> = list_sql.build_query_as().fetch_all(pool).await?;
    Ok(PageResp {
        list,
        total: total as u64,
        page_num,
        page_size,
    })
}
pub async fn page_donate_count_created(
    pool: &Pool<MySql>,
    q: &PageReq,
) -> Result<PageResp<DonateCountCreatedResp>> {
    let page_num = q.page_num.max(1);
    let page_size = q.page_size.clamp(1, 50);
    let offset = (page_num - 1) as u64 * page_size as u64;
    let mut count_sql = QueryBuilder::<MySql>::new(
        "SELECT COUNT(*) as cnt FROM donate_count_created WHERE is_deleted = 0",
    );
    if let Some(pk) = &q.pubkey {
        count_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        count_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        count_sql.push(" AND event_time <= ").push_bind(et);
    }
    let total: i64 = count_sql.build_query_scalar().fetch_one(pool).await?;
    let mut list_sql = QueryBuilder::<MySql>::new(
        r#"
        SELECT
        *
        FROM donate_count_created
        WHERE is_deleted = 0
        "#,
    );
    if let Some(pk) = &q.pubkey {
        list_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        list_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND event_time <= ").push_bind(et);
    }
    list_sql.push(" ORDER BY event_time DESC, id DESC ");
    list_sql.push(" LIMIT ").push_bind(page_size as i64);
    list_sql.push(" OFFSET ").push_bind(offset as i64);
    let list: Vec<DonateCountCreatedResp> = list_sql.build_query_as().fetch_all(pool).await?;
    Ok(PageResp {
        list,
        total: total as u64,
        page_num,
        page_size,
    })
}
pub async fn page_temple_withdrawal(
    pool: &Pool<MySql>,
    q: &PageReq,
) -> Result<PageResp<TempleWithdrawalResp>> {
    let page_num = q.page_num.max(1);
    let page_size = q.page_size.clamp(1, 50);
    let offset = (page_num - 1) as u64 * page_size as u64;
    let mut count_sql = QueryBuilder::<MySql>::new(
        "SELECT COUNT(*) as cnt FROM temple_withdrawal WHERE is_deleted = 0",
    );
    if let Some(pk) = &q.pubkey {
        count_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        count_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        count_sql.push(" AND event_time <= ").push_bind(et);
    }
    let total: i64 = count_sql.build_query_scalar().fetch_one(pool).await?;
    let mut list_sql = QueryBuilder::<MySql>::new(
        r#"
        SELECT
        *
        FROM temple_withdrawal
        WHERE is_deleted = 0
        "#,
    );
    if let Some(pk) = &q.pubkey {
        list_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        list_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND event_time <= ").push_bind(et);
    }
    list_sql.push(" ORDER BY event_time DESC, id DESC ");
    list_sql.push(" LIMIT ").push_bind(page_size as i64);
    list_sql.push(" OFFSET ").push_bind(offset as i64);
    let list: Vec<TempleWithdrawalResp> = list_sql.build_query_as().fetch_all(pool).await?;
    Ok(PageResp {
        list,
        total: total as u64,
        page_num,
        page_size,
    })
}
pub async fn page_like_created(
    pool: &Pool<MySql>,
    q: &PageReq,
) -> Result<PageResp<LikeCreatedResp>> {
    let page_num = q.page_num.max(1);
    let page_size = q.page_size.clamp(1, 50);
    let offset = (page_num - 1) as u64 * page_size as u64;
    let mut count_sql =
        QueryBuilder::<MySql>::new("SELECT COUNT(*) as cnt FROM like_created WHERE is_deleted = 0");
    if let Some(pk) = &q.pubkey {
        count_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        count_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        count_sql.push(" AND event_time <= ").push_bind(et);
    }
    let total: i64 = count_sql.build_query_scalar().fetch_one(pool).await?;
    let mut list_sql = QueryBuilder::<MySql>::new(
        r#"
        SELECT
        *
        FROM like_created
        WHERE is_deleted = 0
        "#,
    );
    if let Some(pk) = &q.pubkey {
        list_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        list_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND event_time <= ").push_bind(et);
    }
    list_sql.push(" ORDER BY event_time DESC, id DESC ");
    list_sql.push(" LIMIT ").push_bind(page_size as i64);
    list_sql.push(" OFFSET ").push_bind(offset as i64);
    let list: Vec<LikeCreatedResp> = list_sql.build_query_as().fetch_all(pool).await?;
    Ok(PageResp {
        list,
        total: total as u64,
        page_num,
        page_size,
    })
}
pub async fn page_wish_created(
    pool: &Pool<MySql>,
    q: &PageReq,
) -> Result<PageResp<WishCreatedResp>> {
    let page_num = q.page_num.max(1);
    let page_size = q.page_size.clamp(1, 50);
    let offset = (page_num - 1) as u64 * page_size as u64;
    let mut count_sql =
        QueryBuilder::<MySql>::new("SELECT COUNT(*) as cnt FROM wish_created WHERE is_deleted = 0");
    if let Some(pk) = &q.pubkey {
        count_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        count_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        count_sql.push(" AND event_time <= ").push_bind(et);
    }
    let total: i64 = count_sql.build_query_scalar().fetch_one(pool).await?;
    let mut list_sql = QueryBuilder::<MySql>::new(
        r#"
        SELECT
        *
        FROM wish_created
        WHERE is_deleted = 0
        "#,
    );
    if let Some(pk) = &q.pubkey {
        list_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        list_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND event_time <= ").push_bind(et);
    }
    list_sql.push(" ORDER BY event_time DESC, id DESC ");
    list_sql.push(" LIMIT ").push_bind(page_size as i64);
    list_sql.push(" OFFSET ").push_bind(offset as i64);
    let list: Vec<WishCreatedResp> = list_sql.build_query_as().fetch_all(pool).await?;
    Ok(PageResp {
        list,
        total: total as u64,
        page_num,
        page_size,
    })
}
pub async fn page_sbt_minted(pool: &Pool<MySql>, q: &PageReq) -> Result<PageResp<SbtMintedResp>> {
    let page_num = q.page_num.max(1);
    let page_size = q.page_size.clamp(1, 50);
    let offset = (page_num - 1) as u64 * page_size as u64;
    let mut count_sql =
        QueryBuilder::<MySql>::new("SELECT COUNT(*) as cnt FROM sbt_minted WHERE is_deleted = 0");
    if let Some(pk) = &q.pubkey {
        count_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        count_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        count_sql.push(" AND event_time <= ").push_bind(et);
    }
    let total: i64 = count_sql.build_query_scalar().fetch_one(pool).await?;
    let mut list_sql = QueryBuilder::<MySql>::new(
        r#"
        SELECT
        *
        FROM sbt_minted
        WHERE is_deleted = 0
        "#,
    );
    if let Some(pk) = &q.pubkey {
        list_sql.push(" AND user = ").push_bind(pk);
    }
    if let Some(st) = &q.start_time {
        list_sql.push(" AND event_time >= ").push_bind(st);
    }
    if let Some(et) = &q.end_time {
        list_sql.push(" AND event_time <= ").push_bind(et);
    }
    list_sql.push(" ORDER BY event_time DESC, id DESC ");
    list_sql.push(" LIMIT ").push_bind(page_size as i64);
    list_sql.push(" OFFSET ").push_bind(offset as i64);
    let list: Vec<SbtMintedResp> = list_sql.build_query_as().fetch_all(pool).await?;
    Ok(PageResp {
        list,
        total: total as u64,
        page_num,
        page_size,
    })
}
