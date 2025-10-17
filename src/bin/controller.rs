use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use sol_ji_service::{
    domain::{PageReq, TimeReq},
    infra::{
        connect_db, get_temple_by_admin, get_user_info_by_pubkey, page_coin_flip, page_donate,
        page_donate_count_created, page_draw_lots, page_incense_bought, page_incense_burned,
        page_like_created, page_medal_minted, page_medal_upgraded, page_nft_destroy,
        page_sbt_minted, page_temple_withdrawal, page_user_info, page_wish_created,
        query_interactions_count, query_user_activity, query_user_count,
        query_user_donation_ranking, query_user_merit_ranking,
    },
    utils::ApiResponse,
};
use sqlx::{MySql, Pool};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8888);

    let pool: Pool<MySql> = connect_db().await.expect("connect db");

    println!("API listening on http://{host}:{port}");

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive()) // 开发期跨域
            .app_data(web::Data::new(pool.clone())) // 传入连接池
            .service(query_temple_handler)
            .service(query_user_info_handler)
            .service(query_user_info_page_handler)
            .service(query_user_count_handler)
            .service(query_interactions_count_handler)
            .service(query_user_donation_ranking_handler)
            .service(query_user_merit_ranking_handler)
            .service(query_wish_created_page_handler)
            .service(query_user_activity_handler)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}

/// 寺庙信息
#[get("/queryTemple/{pubkey}")]
pub async fn query_temple_handler(
    path: web::Path<String>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    let pubkey: String = path.into_inner();
    match get_temple_by_admin(pool.get_ref(), &pubkey).await {
        Ok(Some(row)) => Ok(HttpResponse::Ok().json(ApiResponse::ok(row))),
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::not_found("temple not found")))
        }
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

// user info
#[get("queryUserInfo/{pubkey}")]
pub async fn query_user_info_handler(
    path: web::Path<String>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    let pubkey: String = path.into_inner();
    match get_user_info_by_pubkey(pool.get_ref(), &pubkey).await {
        Ok(Some(row)) => Ok(HttpResponse::Ok().json(ApiResponse::ok(row))),
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::not_found("user info not found")))
        }
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

#[get("queryUserInfoPage")]
pub async fn query_user_info_page_handler(
    query: web::Query<PageReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match page_user_info(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}
/// user count
// 条件不传递查询所有信徒,传递时间查阅活跃用户数
#[get("queryUserCount")]
pub async fn query_user_count_handler(
    query: web::Query<TimeReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match query_user_count(pool.get_ref(), &query).await {
        Ok(count) => Ok(HttpResponse::Ok().json(ApiResponse::ok(count))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

// interactions count  互动次数
#[get("queryInteractionsCount/{pubkey}")]
pub async fn query_interactions_count_handler(
    path: web::Path<String>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    let pubkey: String = path.into_inner();
    match query_interactions_count(pool.get_ref(), &pubkey).await {
        Ok(Some(row)) => Ok(HttpResponse::Ok().json(ApiResponse::ok(row))),
        Ok(None) => {
            Ok(HttpResponse::NotFound().json(ApiResponse::<()>::not_found("temple not found")))
        }
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

// User donation ranking
#[get("queryUserDonationRanking")]
pub async fn query_user_donation_ranking_handler(
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match query_user_donation_ranking(pool.get_ref()).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

#[get("queryUserMeritRanking")]
pub async fn query_user_merit_ranking_handler(
    query: web::Query<TimeReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match query_user_merit_ranking(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

// // active user count
#[get("queryUserActivety")]
pub async fn query_user_activity_handler(
    query: web::Query<TimeReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match query_user_activity(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

#[get("querIncenseBoughtPage")]
pub async fn query_incense_bought_page_handler(
    query: web::Query<PageReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match page_incense_bought(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

#[get("querIncenseBurnedPage")]
pub async fn query_incense_burned_page_handler(
    query: web::Query<PageReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match page_incense_burned(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

#[get("querDonatePage")]
pub async fn query_donate_page_handler(
    query: web::Query<PageReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match page_donate(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

#[get("querMedalMintedPage")]
pub async fn query_medal_minted_page_handler(
    query: web::Query<PageReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match page_medal_minted(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}
#[get("querMedalUpgradedPage")]
pub async fn query_medal_upgraded_page_handler(
    query: web::Query<PageReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match page_medal_upgraded(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

#[get("querDrawLotsPage")]
pub async fn query_draw_lots_page_handler(
    query: web::Query<PageReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match page_draw_lots(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

#[get("querCoinFlipPage")]
pub async fn query_coin_flip_page_handler(
    query: web::Query<PageReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match page_coin_flip(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

#[get("querNftDestoryPage")]
pub async fn query_nft_destroy_page_handler(
    query: web::Query<PageReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match page_nft_destroy(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

#[get("queryDonateCountCreatedPage")]
pub async fn query_donate_count_created_page_handler(
    query: web::Query<PageReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match page_donate_count_created(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

#[get("queryTempleWithdrawalPage")]
pub async fn query_temple_withdrawal_page_handler(
    query: web::Query<PageReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match page_temple_withdrawal(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

#[get("queryLikeCreatedPage")]
pub async fn query_like_created_page_handler(
    query: web::Query<PageReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match page_like_created(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

#[get("queryWishCreatedPage")]
pub async fn query_wish_created_page_handler(
    query: web::Query<PageReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match page_wish_created(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}

#[get("querySbtMintedPage")]
pub async fn query_sbt_minted_page_handler(
    query: web::Query<PageReq>,
    pool: web::Data<Pool<MySql>>,
) -> actix_web::Result<impl Responder> {
    match page_sbt_minted(pool.get_ref(), &query).await {
        Ok(resp) => Ok(HttpResponse::Ok().json(ApiResponse::ok(resp))),
        Err(e) => {
            eprintln!("query error: {e:?}");
            Ok(HttpResponse::InternalServerError().json(ApiResponse::<()>::err("internal error")))
        }
    }
}
