use dotenv::dotenv;

mod bot;
use bot::bot::bot_main;

mod dl;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    bot_main().await?;
    Ok(())
}
