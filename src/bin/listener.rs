use dotenvy::dotenv;
use sol_ji_service::{business::run_logs, infra::connect_db};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    println!("DATABASE_URL = {:?}", std::env::var("DATABASE_URL")?);
    let pool = connect_db().await?;
    run_logs(&pool).await
}
