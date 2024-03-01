use anyhow;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Sqlite, SqlitePool};
use std::env;
use std::fmt;
use std::str;
use std::str::FromStr;
use std::time::Duration;
use teloxide::dispatching::dialogue;
use teloxide::dispatching::dialogue::InMemStorage;
use teloxide::dispatching::UpdateHandler;
use teloxide::types::InputFile;
use teloxide::{prelude::*, update_listeners::Polling, utils::command::BotCommands};
use tracing::{event, Level};

use super::log::log_init;
use super::util::make_database_url;

use crate::dl::delete_if_exists;
use crate::dl::download;

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

pub async fn bot_main() -> anyhow::Result<()> {
    log_init();
    event!(Level::INFO, "start");

    let db_url = make_database_url();
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        Sqlite::create_database(&db_url)
            .await
            .expect("failed to create database");
    }

    let db = SqlitePool::connect(&db_url).await?;
    sqlx::migrate!().run(&db).await?;

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
        .branch(case![Command::Download(url)].endpoint(cmd_download));

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

#[derive(sqlx::FromRow, Debug)]
struct DbUser {
    pub id: i64,
    pub tg_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub can_download: i64,
    pub is_admin: i64
}

async fn cmd_test(bot: Bot, msg: Message, db: SqlitePool) -> HandlerResult {
    bot.send_message(msg.chat.id, "test response").await?;

    let user = msg.from().unwrap();
    
    let conn = db.acquire().await?;
    sqlx::query("INSERT OR IGNORE user
        (tg_id, user_name, first_name, last_name, can_download, is_admin)
        VALUES ($1, $2, $3, $4, $5, $6);")
        .bind(user.id.0 as i64)
        .bind(&user.username)
        .bind(&user.first_name)
        .bind(&user.last_name)
        .bind(0)
        .bind(0)
        .execute(&db).await.expect("insert");

    let db_user = sqlx::query_as!(DbUser, 
        "SELECT * FROM user WHERE id = 1 LIMIT 1;")
        .fetch_one(&db).await.expect("fetch_one");
    dbg!(db_user);
    Ok(())
}

async fn bot_download(bot: Bot, msg: Message, url: String) -> HandlerResult {
    let output_path = match download(url.as_str()).await {
        Ok(path) => path,
        Err(e) => {
            event!(Level::ERROR, "{}", e.to_string());
            bot.send_message(msg.chat.id, e.to_string()).await?;
            return Ok(());
        }
    };

    if let Err(e) = bot
        .send_video(msg.chat.id, InputFile::file(&output_path))
        .await
    {
        delete_if_exists(&output_path);
        return Err(Box::new(e));
    }

    Ok(())
}

async fn cmd_download(bot: Bot, msg: Message, url: String) -> HandlerResult {
    bot_download(bot, msg, url).await
}

async fn handle_message(_bot: Bot, _dialogue: MyDialogue, _msg: Message) -> HandlerResult {
    Ok(())
}
