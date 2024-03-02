use sqlx::migrate::MigrateDatabase;
use sqlx::{Sqlite, SqlitePool};

use super::util::make_database_url;

pub type DbPool = SqlitePool;

#[derive(sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub tg_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub can_download: i64,
    pub is_admin: i64,
}

pub mod user;

#[derive(sqlx::FromRow)]
pub struct Chat {
    pub id: i64,
    pub tg_id: i64,
    pub username: Option<String>,
    pub title: String,
    pub can_download: i64,
}

#[derive(sqlx::FromRow)]
pub struct Link {
    pub id: i64,
    pub domain: String,
    pub path: Option<String>,
    pub download_allowed: i64,
    pub auto_download: i64,
}

#[derive(sqlx::FromRow)]
pub struct Request {
    pub id: i64,
    pub requested_by: i64,
    pub approved_by: Option<i64>,
    pub message: Option<String>,
    pub is_approved: i64,
}

pub async fn db_init() -> SqlitePool {
    let db_url = make_database_url();
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        Sqlite::create_database(&db_url)
            .await
            .expect("failed to create database");
    }

    let db = SqlitePool::connect(&db_url).await.unwrap();
    sqlx::migrate!().run(&db).await.unwrap();

    db
}
