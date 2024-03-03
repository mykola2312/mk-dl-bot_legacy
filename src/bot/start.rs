use rust_i18n::t;
use teloxide::prelude::*;
use teloxide::types::Me;
use tracing::{event, Level};

use super::types::HandlerResult;
use crate::db::chat::find_or_create_chat;
use crate::db::user::find_or_create_user;
use crate::db::DbPool;

pub async fn cmd_start(bot: Bot, msg: Message, db: DbPool) -> HandlerResult {
    if msg.chat.is_private() {
        if let Some(user) = msg.from() {
            let user = find_or_create_user(&db, user).await?;
            sqlx::query("UPDATE user SET has_private_chat = 1 WHERE id = $1;")
                .bind(user.id)
                .execute(&db)
                .await?;

            event!(
                Level::INFO,
                "user {} has started private chat with bot",
                user
            );
            bot.send_message(msg.chat.id, t!("started_private_chat"))
                .await?;
        }
    } else if msg.chat.is_channel()
        || msg.chat.is_chat()
        || msg.chat.is_group()
        || msg.chat.is_supergroup()
    {
        let chat = find_or_create_chat(&db, &msg.chat).await?;
        event!(Level::INFO, "started public chat {}", chat);
        bot.send_message(msg.chat.id, t!("started_public_chat"))
            .await?;
    }
    Ok(())
}

pub async fn handle_my_chat_member(db: DbPool, upd: ChatMemberUpdated) {
    match find_or_create_chat(&db, &upd.chat).await {
        Ok(chat) => event!(Level::INFO, "started public chat {}", chat),
        Err(e) => event!(Level::ERROR, "{}", e),
    }
}

pub async fn handle_new_chat_member(
    bot: Bot,
    tg_chat: &teloxide::types::Chat,
    new: Vec<teloxide::types::User>,
    db: DbPool,
    me: Me,
) -> HandlerResult {
    for member in new {
        if member.id == me.id {
            // We've been added in chat
            let chat = find_or_create_chat(&db, tg_chat).await?;
            event!(Level::INFO, "started public chat {}", chat);
            bot.send_message(tg_chat.id, t!("started_public_chat"))
                .await?;
        }
    }

    Ok(())
}
