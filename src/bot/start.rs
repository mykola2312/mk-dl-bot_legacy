use sqlx::Row;
use teloxide::prelude::*;
use tracing::{event, Level};

use super::types::HandlerResult;
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

            event!(Level::INFO, "user {} has started private chat with bot", user);
            bot.send_message(msg.chat.id, "Since you've initiated private chat now you could receive messages from bot").await?;
        }
    }
    Ok(())
}
