use teloxide::types;

use super::{DbPool, User};
use crate::unwrap_or_create;

pub async fn create_user(db: &DbPool, user: &types::User) -> Result<User, sqlx::Error> {
    sqlx::query(
        r#"INSERT OR IGNORE INTO "user"
        (tg_id, username, first_name, last_name, can_download, is_admin, has_private_chat)
        VALUES ($1,$2,$3,$4,$5,$6,$7);"#,
    )
    .bind(user.id.0 as i64)
    .bind(&user.username)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(false)
    .bind(false)
    .bind(0)
    .execute(db)
    .await?;

    let user: User = sqlx::query_as(r#"SELECT * FROM "user" WHERE tg_id = $1 LIMIT 1;"#)
        .bind(user.id.0 as i64)
        .fetch_one(db)
        .await?;
    Ok(user)
}

pub async fn find_or_create_user(db: &DbPool, user: &types::User) -> Result<User, sqlx::Error> {
    let res: Result<User, sqlx::Error> =
        sqlx::query_as(r#"SELECT * FROM "user" WHERE tg_id = $1 LIMIT 1;"#)
            .bind(user.id.0 as i64)
            .fetch_one(db)
            .await;

    unwrap_or_create!(db, user, res, create_user)
}
