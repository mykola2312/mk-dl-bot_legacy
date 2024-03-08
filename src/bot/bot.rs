use anyhow;
use rust_i18n::t;
use std::str;
use std::time::Duration;
use teloxide::dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler};
use teloxide::types::{Me, MessageKind, MessageNewChatMembers, UpdateKind};
use teloxide::{prelude::*, update_listeners::Polling, utils::command::BotCommands};
use tracing::{event, Level};

use super::start::handle_new_chat_member;
use super::types::*;
use crate::db::DbPool;
use crate::util::{unwrap_env, parse_env};

use super::dl::cmd_download;
use super::op::cmd_op;
use super::request::{cmd_approve, cmd_decline, cmd_listrequests, cmd_request};
use super::request_chat::{
    cmd_approve_chat, cmd_decline_chat, cmd_listrequests_chat, cmd_request_chat,
};
use super::start::{cmd_start, handle_my_chat_member};

pub async fn bot_main(db: DbPool) -> anyhow::Result<()> {
    event!(Level::INFO, "start");

    let bot = Bot::new(unwrap_env("BOT_TOKEN"));
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
        .branch(case![Command::Start].endpoint(cmd_start))
        .branch(case![Command::Download(url)].endpoint(cmd_download))
        .branch(case![Command::OP].endpoint(cmd_op))
        .branch(case![Command::Request(text)].endpoint(cmd_request))
        .branch(case![Command::ListRequests].endpoint(cmd_listrequests))
        .branch(case![Command::Approve(text)].endpoint(cmd_approve))
        .branch(case![Command::Decline(text)].endpoint(cmd_decline))
        .branch(case![Command::RequestChat(text)].endpoint(cmd_request_chat))
        .branch(case![Command::ListRequestsChat].endpoint(cmd_listrequests_chat))
        .branch(case![Command::ApproveChat(text)].endpoint(cmd_approve_chat))
        .branch(case![Command::DeclineChat(text)].endpoint(cmd_decline_chat));

    let message_handler = Update::filter_message().branch(command_handler);
    let raw_message_handler = Update::filter_message().branch(dptree::endpoint(handle_message));

    dialogue::enter::<Update, InMemStorage<()>, (), _>()
        .branch(message_handler)
        .branch(raw_message_handler)
        .endpoint(handle_update)
}

async fn handle_update(_bot: Bot, upd: Update, db: DbPool) -> HandlerResult {
    match upd.kind {
        UpdateKind::MyChatMember(upd) => handle_my_chat_member(db, upd).await,
        _ => event!(Level::WARN, "unhandled update {:?}", upd),
    }

    Ok(())
}

async fn handle_message(
    bot: Bot,
    _dialogue: MyDialogue,
    msg: Message,
    db: DbPool,
    me: Me,
) -> HandlerResult {
    match msg.kind {
        MessageKind::NewChatMembers(MessageNewChatMembers { new_chat_members }) => {
            handle_new_chat_member(bot, &msg.chat, new_chat_members, db, me).await?
        }
        MessageKind::Common(_) => (),
        _ => {
            dbg!(msg);
        }
    }

    Ok(())
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase")]
enum Command {
    Test,
    #[command(alias = "start")]
    Start,
    #[command(alias = "dl")]
    Download(String),
    #[command(alias = "op")]
    OP,

    #[command(alias = "request")]
    Request(String),
    #[command(alias = "listrequests")]
    ListRequests,
    #[command(alias = "approve")]
    Approve(String),
    #[command(alias = "decline")]
    Decline(String),

    #[command(alias = "request_chat")]
    RequestChat(String),
    #[command(alias = "listrequests_chat")]
    ListRequestsChat,
    #[command(alias = "approve_chat")]
    ApproveChat(String),
    #[command(alias = "decline_chat")]
    DeclineChat(String),
}

async fn cmd_test(bot: Bot, msg: Message, _db: DbPool) -> HandlerResult {
    bot.send_message(msg.chat.id, t!("test_response")).await?;
    dbg!(msg);

    Ok(())
}
