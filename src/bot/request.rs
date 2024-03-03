use rust_i18n::t;
use sqlx::Row;
use teloxide::prelude::*;
use tracing::{event, Level};

use super::notify::notify_admins;
use super::types::HandlerResult;
use crate::db::chat::find_or_create_chat;
use crate::db::user::find_or_create_user;
use crate::db::DbPool;
use crate::reply_i18n_and_return;

pub async fn cmd_request(bot: Bot, msg: Message, text: String, db: DbPool) -> HandlerResult {
    if text.len() < 16 {
        reply_i18n_and_return!(bot, msg.chat.id, "request_text_is_too_short");
    } else if text.len() > 100 {
        reply_i18n_and_return!(bot, msg.chat.id, "request_text_is_too_long");
    }

    if let Some(user) = msg.from() {
        let user = find_or_create_user(&db, user).await?;
        if user.can_download == 1 {
            reply_i18n_and_return!(bot, msg.chat.id, "already_can_download");
        }

        let requests: i64 = sqlx::query("SELECT COUNT(1) FROM request WHERE requested_by = $1;")
            .bind(user.id)
            .fetch_one(&db)
            .await?
            .get(0);
        if requests > 0 {
            reply_i18n_and_return!(bot, msg.chat.id, "already_has_requested");
        }

        // put the request
        sqlx::query("INSERT INTO request (requested_by,message,is_approved) VALUES ($1,$2,$3);")
            .bind(user.id)
            .bind(text)
            .bind(0)
            .execute(&db)
            .await?;
        event!(Level::INFO, "added request for {}", user);

        // notify admins
        notify_admins(
            &bot,
            &db,
            t!("admin_notify_request", user = user.to_string()).to_string(),
        )
        .await?;
        
        bot.send_message(msg.chat.id, t!("request_added")).await?;
    }

    Ok(())
}
