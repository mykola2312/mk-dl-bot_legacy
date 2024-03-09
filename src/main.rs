use dotenv::dotenv;

mod bot;
use bot::bot::bot_main;

mod dl;

mod util;

mod log;
use log::log_init;

mod db;
use db::db_init;

rust_i18n::i18n!("locales");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if cfg!(debug_assertions) {
        dotenv::from_filename(".env.dev").ok();
    }

    log_init();
    let db = db_init().await;

    bot_main(db).await?;
    Ok(())
}
