use teloxide::types;

use super::{DbPool, User};

pub async fn create_user(
    db: &DbPool,
    user: &types::User,
    can_download: bool,
    is_admin: bool,
) -> Result<User, sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO user
        (tg_id, username, first_name, last_name, can_download, is_admin)
        VALUES ($1,$2,$3,$4,$5,$6);",
    )
    .bind(user.id.0 as i64)
    .bind(&user.username)
    .bind(&user.first_name)
    .bind(&user.last_name)
    .bind(can_download as i64)
    .bind(is_admin as i64)
    .execute(db.as_ref())
    .await?;

    let user: User = sqlx::query_as("SELECT * FROM user WHERE tg_id = $1 LIMIT 1;")
        .bind(user.id.0 as i64)
        .fetch_one(db.as_ref())
        .await?;
    Ok(user)
}

pub async fn find_or_create_user(db: &DbPool, user: &types::User) -> Result<User, sqlx::Error> {
    let res: Result<User, sqlx::Error> =
        sqlx::query_as("SELECT * FROM user WHERE tg_id = $1 LIMIT 1;")
            .bind(user.id.0 as i64)
            .fetch_one(db.as_ref())
            .await;

    match res {
        Ok(user) => return Ok(user),
        Err(e) => match e {
            sqlx::Error::RowNotFound => create_user(db, user, false, false).await,
            _ => Err(e),
        },
    }
}
