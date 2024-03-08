use teloxide::types;

use super::{Chat, DbPool};
use crate::unwrap_or_create;

pub async fn create_chat(db: &DbPool, chat: &types::Chat) -> Result<Chat, sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO "chat" (tg_id,title,username,can_download)
        VALUES ($1,$2,$3,$4)"#,
    )
    .bind(chat.id.0 as i64)
    .bind(chat.title())
    .bind(chat.username())
    .bind(false)
    .execute(db)
    .await?;

    let chat: Chat = sqlx::query_as(r#"SELECT * FROM "chat" WHERE tg_id = $1 LIMIT 1;"#)
        .bind(chat.id.0 as i64)
        .fetch_one(db)
        .await?;
    Ok(chat)
}

pub async fn find_or_create_chat(db: &DbPool, chat: &types::Chat) -> Result<Chat, sqlx::Error> {
    let res: Result<Chat, sqlx::Error> =
        sqlx::query_as(r#"SELECT * FROM "chat" WHERE tg_id = $1 LIMIT 1;"#)
            .bind(chat.id.0 as i64)
            .fetch_one(db)
            .await;

    unwrap_or_create!(db, chat, res, create_chat)
}
