use std::sync::Arc;

use dotenv::dotenv;

mod bot;
use bot::bot::bot_main;

mod dl;

mod util;

mod log;
use log::log_init;

mod db;
use db::db_init;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    log_init();
    let db = db_init().await;

    bot_main(Arc::from(db)).await?;
    Ok(())
}
