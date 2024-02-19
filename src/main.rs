use anyhow;
use dotenv::dotenv;
use std::env;
use std::fmt;
use std::str;
use std::str::FromStr;
use std::time::Duration;
use teloxide::dispatching::dialogue;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::UpdateHandler;
use teloxide::{prelude::*, update_listeners::Polling, utils::command::BotCommands};

mod dl;
use dl::yt_dlp::YtDlp;

type State = ();
type MyDialogue = Dialogue<State, InMemStorage<State>>;

type HandlerErr = Box<dyn std::error::Error + Send + Sync>;
type HandlerResult = Result<(), HandlerErr>;

fn parse_env<T>(name: &str) -> T
where
    T: FromStr,
    T::Err: fmt::Debug,
{
    str::parse(
        env::var(name)
            .expect(format!("env '{}' variable not defined", name).as_str())
            .as_str(),
    )
    .expect(format!("env '{}' parse error", name).as_str())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    let _ = YtDlp::load_info(env::var("TEST_URL")?.as_str()).await;

    Ok(())
    //bot_main().await
}

async fn bot_main() -> anyhow::Result<()> {
    let bot = Bot::new(env::var("BOT_TOKEN")?);
    let listener = Polling::builder(bot.clone())
        .timeout(Duration::from_secs(parse_env("POLLING_TIMEOUT")))
        .limit(parse_env("POLLING_LIMIT"))
        .drop_pending_updates()
        .build();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![InMemStorage::<State>::new()])
        .enable_ctrlc_handler()
        .build()
        .dispatch_with_listener(
            listener,
            LoggingErrorHandler::with_custom_text("update listener error"),
        )
        .await;

    Ok(())
}

fn schema() -> UpdateHandler<HandlerErr> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(case![Command::Test].endpoint(test))
        .branch(case![Command::Download(url)].endpoint(download));

    let message_handler = Update::filter_message().branch(command_handler);
    let raw_message_handler = Update::filter_message().branch(dptree::endpoint(handle_message));

    dialogue::enter::<Update, InMemStorage<()>, (), _>()
        .branch(message_handler)
        .branch(raw_message_handler)
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    Test,

    #[command(alias = "dl")]
    Download(String),
}

async fn test(bot: Bot, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "test response").await?;

    Ok(())
}

async fn download(bot: Bot, msg: Message, url: String) -> HandlerResult {
    Ok(())
}

async fn handle_message(_bot: Bot, _dialogue: MyDialogue, msg: Message) -> HandlerResult {
    println!(
        "msg {} kind {:?} text {}",
        msg.id,
        msg.kind,
        msg.text().unwrap_or("<empty>")
    );

    Ok(())
}
