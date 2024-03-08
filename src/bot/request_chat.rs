use rust_i18n::t;
use sqlx::Row;
use teloxide::prelude::*;
use teloxide::types::Recipient;
use tracing::{event, Level};

use super::notify::notify_admins;
use super::types::HandlerResult;
use crate::db::chat::find_or_create_chat;
use crate::db::user::find_or_create_user;
use crate::db::{Chat, DbPool};
use crate::{parse_integer, reply_i18n_and_return};

pub async fn cmd_request_chat(bot: Bot, msg: Message, text: String, db: DbPool) -> HandlerResult {
    if text.len() < 16 {
        reply_i18n_and_return!(bot, msg.chat.id, "request_text_is_too_short");
    } else if text.len() > 100 {
        reply_i18n_and_return!(bot, msg.chat.id, "request_text_is_too_long");
    }

    if let Some(user) = msg.from() {
        let user = find_or_create_user(&db, user).await?;
        let chat = find_or_create_chat(&db, &msg.chat).await?;

        if chat.can_download {
            reply_i18n_and_return!(bot, msg.chat.id, "chat_already_can_download");
        }

        let requests: i64 =
            sqlx::query(r#"SELECT COUNT(1) FROM "request_chat" WHERE requested_for = $1;"#)
                .bind(chat.id)
                .fetch_one(&db)
                .await?
                .get(0);
        if requests > 0 {
            reply_i18n_and_return!(bot, msg.chat.id, "chat_already_has_requested");
        }

        // put the chat request
        sqlx::query(r#"INSERT INTO "request_chat" (requested_by,requested_for,message,is_approved) VALUES ($1,$2,$3,$4);"#)
            .bind(user.id)
            .bind(chat.id)
            .bind(text)
            .bind(false)
            .execute(&db)
            .await?;
        event!(Level::INFO, "added chat request for {}", chat);

        // notify admins
        notify_admins(
            &bot,
            &db,
            t!("admin_notify_chat_request", chat = chat.to_string()).to_string(),
        )
        .await?;

        bot.send_message(msg.chat.id, t!("chat_request_added"))
            .await?;
    }

    Ok(())
}

#[derive(sqlx::FromRow, Debug)]
struct RequestChatWithChat {
    pub request_id: i32,
    pub message: String,
    #[sqlx(flatten)]
    pub chat: Chat,
}

pub async fn cmd_listrequests_chat(bot: Bot, msg: Message, db: DbPool) -> HandlerResult {
    if let Some(user) = msg.from() {
        let user = find_or_create_user(&db, user).await?;
        if !user.is_admin {
            reply_i18n_and_return!(bot, msg.chat.id, "not_an_admin");
        }

        let requests: Vec<RequestChatWithChat> = sqlx::query_as(
            r#"SELECT "request_chat".id AS request_id, "request_chat".message, "chat".*
            FROM "request_chat"
            INNER JOIN "chat" ON "request_chat".requested_for = "chat".id
            WHERE "request_chat".is_approved = false;"#,
        )
        .fetch_all(&db)
        .await?;

        let mut list = String::new();
        list.push_str(t!("chat_request_list_header").to_string().as_str());
        for request in requests {
            let fmt = format!(
                "{}: {}: {}\n",
                request.request_id, request.chat, request.message
            );
            list.push_str(fmt.as_str());
        }
        bot.send_message(msg.chat.id, list).await?;
    }

    Ok(())
}

pub async fn cmd_approve_chat(bot: Bot, msg: Message, id: String, db: DbPool) -> HandlerResult {
    let id: i32 = parse_integer!(bot, msg.chat.id, id);

    if let Some(user) = msg.from() {
        let user = find_or_create_user(&db, user).await?;
        if !user.is_admin {
            reply_i18n_and_return!(bot, msg.chat.id, "not_an_admin");
        }

        // get request
        let res: Result<RequestChatWithChat, sqlx::Error> = sqlx::query_as(
            r#"SELECT "request_chat".id AS request_id, "request_chat".message, "chat".*
            FROM "request_chat"
            INNER JOIN "chat" ON "request_chat".requested_for = "chat".id
            WHERE "request_chat".id = $1 AND "request_chat".is_approved = false
            LIMIT 1;"#,
        )
        .bind(id)
        .fetch_one(&db)
        .await;
        let request = match res {
            Ok(request) => request,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    bot.send_message(msg.chat.id, t!("chat_request_not_found"))
                        .await?;
                    return Ok(());
                }
                _ => return Err(Box::new(e)),
            },
        };

        // approve request
        sqlx::query(r#"UPDATE "request_chat" SET approved_by = $1, is_approved = true WHERE id = $2;"#)
            .bind(user.id)
            .bind(request.request_id)
            .execute(&db)
            .await?;
        event!(
            Level::INFO,
            "approved chat request {} by {} for {}",
            request.request_id,
            user,
            request.chat
        );
        // notify target chat
        bot.send_message(
            Recipient::Id(ChatId(request.chat.tg_id)),
            t!("chat_request_approved"),
        )
        .await?;
    }

    Ok(())
}

pub async fn cmd_decline_chat(bot: Bot, msg: Message, id: String, db: DbPool) -> HandlerResult {
    let id: i32 = parse_integer!(bot, msg.chat.id, id);

    if let Some(user) = msg.from() {
        let user = find_or_create_user(&db, user).await?;
        if !user.is_admin {
            reply_i18n_and_return!(bot, msg.chat.id, "not_an_admin");
        }

        // get request
        let res: Result<RequestChatWithChat, sqlx::Error> = sqlx::query_as(
            r#"SELECT "request_chat".id AS request_id, "request_chat".message, "chat".*
            FROM "request_chat"
            INNER JOIN "chat" ON "request_chat".requested_for = "chat".id
            WHERE "request_chat".id = $1 AND "request_chat".is_approved = false
            LIMIT 1;"#,
        )
        .bind(id)
        .fetch_one(&db)
        .await;
        let request = match res {
            Ok(request) => request,
            Err(e) => match e {
                sqlx::Error::RowNotFound => {
                    bot.send_message(msg.chat.id, t!("chat_request_not_found"))
                        .await?;
                    return Ok(());
                }
                _ => return Err(Box::new(e)),
            },
        };

        // decline request
        sqlx::query(r#"DELETE FROM request_chat WHERE id = $1;"#)
            .bind(request.request_id)
            .execute(&db)
            .await?;
        event!(
            Level::INFO,
            "declined request {} by {} for {}",
            request.request_id,
            user,
            request.chat
        );
        bot.send_message(msg.chat.id, t!("request_declined"))
            .await?;

        // notify target chat
        bot.send_message(
            Recipient::Id(ChatId(request.chat.tg_id)),
            t!("chat_request_declined"),
        )
        .await?;
    }

    Ok(())
}
