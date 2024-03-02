use anyhow;
use sqlx::SqlitePool;
use std::env;
use std::fmt;
use std::str;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use teloxide::dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler};
use teloxide::types::Recipient;
use teloxide::{prelude::*, update_listeners::Polling, utils::command::BotCommands};
use tracing::{event, Level};

use super::types::*;
use crate::db::DbPool;

use super::dl::cmd_download;
use super::op::cmd_op;

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

pub async fn bot_main(db: DbPool) -> anyhow::Result<()> {
    event!(Level::INFO, "start");

    let bot = Bot::new(env::var("BOT_TOKEN")?);
    let listener = Polling::builder(bot.clone())
        .timeout(Duration::from_secs(parse_env("POLLING_TIMEOUT")))
        .limit(parse_env("POLLING_LIMIT"))
        .drop_pending_updates()
        .build();

    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![db, InMemStorage::<State>::new()])
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
        .branch(case![Command::Test].endpoint(cmd_test))
        .branch(case![Command::Download(url)].endpoint(cmd_download))
        .branch(case![Command::OP].endpoint(cmd_op));

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

    #[command(alias = "op")]
    OP,
}

async fn cmd_test(bot: Bot, msg: Message, _db: DbPool) -> HandlerResult {
    bot.send_message(msg.chat.id, "test response").await?;

    Ok(())
}

async fn handle_message(_bot: Bot, _dialogue: MyDialogue, _msg: Message) -> HandlerResult {
    Ok(())
}
