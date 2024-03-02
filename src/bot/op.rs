use sqlx::Row;
use teloxide::prelude::*;
use tracing::{event, Level};

use super::types::HandlerResult;
use crate::db::user::{create_user, find_or_create_user};
use crate::db::DbPool;

pub async fn cmd_op(bot: Bot, msg: Message, db: DbPool) -> HandlerResult {
    let admins: i64 = sqlx::query("SELECT COUNT(*) FROM user WHERE is_admin = 1")
        .fetch_one(&db)
        .await?
        .get(0);

    if let Some(tg_user) = msg.from() {
        if admins == 0 {
            let user = create_user(&db, tg_user, true, true).await?;

            event!(
                Level::INFO,
                "opped {} - {}",
                user.tg_id,
                user.username_or_name()
            );
            bot.send_message(msg.chat.id, "Now you're an admin").await?;
        } else {
            let user = find_or_create_user(&db, tg_user).await?;
            if user.is_admin == 1 {
                if let Some(target) = msg.reply_to_message().and_then(|m| m.from()) {
                    let target = find_or_create_user(&db, target).await?;
                    sqlx::query("UPDATE user SET can_download = 1, is_admin = 1 WHERE id = $1;")
                        .bind(target.id)
                        .execute(&db)
                        .await?;

                    event!(
                        Level::INFO,
                        "opped {} - {}",
                        target.tg_id,
                        target.username_or_name()
                    );
                    bot.send_message(msg.chat.id, "opped").await?;
                } else {
                    bot.send_message(msg.chat.id, "You have to reply on target's message")
                        .await?;
                }
            } else {
                bot.send_message(msg.chat.id, "You can't do that bruh")
                    .await?;
            }
        }
    }

    Ok(())
}
