use rust_i18n::t;
use sqlx::Row;
use teloxide::{prelude::*, requests};
use tracing::{event, Level};

use super::notify::notify_admins;
use super::types::HandlerResult;
use crate::db::chat::find_or_create_chat;
use crate::db::user::find_or_create_user;
use crate::db::{DbPool, User};
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

#[derive(sqlx::FromRow, Debug)]
struct RequestWithUser {
    pub request_id: i64,
    pub message: String,
    #[sqlx(flatten)]
    pub user: User,
}

pub async fn cmd_listrequests(bot: Bot, msg: Message, db: DbPool) -> HandlerResult {
    if let Some(user) = msg.from() {
        let user = find_or_create_user(&db, user).await?;
        if user.is_admin != 1 {
            reply_i18n_and_return!(bot, msg.chat.id, "not_an_admin");
        }

        let requests: Vec<RequestWithUser> = sqlx::query_as(
            "SELECT request.id AS request_id, request.message, user.*
            FROM request
            INNER JOIN user	ON request.requested_by = user.id
            WHERE request.is_approved = 0;",
        )
        .fetch_all(&db)
        .await?;
        
        let mut list = String::new();
        list.push_str(t!("request_list_header").to_string().as_str());
        for request in requests {
            let fmt = format!(
                "{}: {}: {}\n",
                request.request_id, request.user, request.message
            );
            list.push_str(fmt.as_str());
        }
        bot.send_message(msg.chat.id, list).await?;
    }

    Ok(())
}
