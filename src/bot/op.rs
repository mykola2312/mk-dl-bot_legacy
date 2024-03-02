use sqlx::Row;
use teloxide::prelude::*;
use tracing::{event, Level};

use super::types::HandlerResult;
use crate::db::{user::create_user, DbPool};

pub async fn cmd_op(bot: Bot, msg: Message, db: DbPool) -> HandlerResult {
    let admins: i64 = sqlx::query("SELECT COUNT(*) FROM user WHERE is_admin = 1")
        .fetch_one(&db)
        .await?
        .get(0);

    if let Some(user) = msg.from() {
        if admins == 0 {
            let user = create_user(db, user, true, true).await?;
            event!(
                Level::INFO,
                "opped {} - {}",
                user.tg_id,
                user.username.unwrap_or(user.first_name)
            );
            bot.send_message(msg.chat.id, "Now you're an admin").await?;
        } else {
            bot.send_message(msg.chat.id, "You can't do that anymore").await?;
        }
    }

    Ok(())
}
