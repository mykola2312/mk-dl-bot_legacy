use sqlx::migrate::MigrateDatabase;
use sqlx::{Sqlite, SqlitePool};
use std::fmt;

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
    pub has_private_chat: i64,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.tg_id, self.username_or_name())
    }
}

impl User {
    pub fn username_or_name(&self) -> &String {
        self.username.as_ref().unwrap_or(&self.first_name)
    }
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

pub mod chat;

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

#[macro_export]
macro_rules! unwrap_or_create {
    ($db:expr, $tg:expr, $res:expr, $create:expr) => {
        match $res {
            Ok(obj) => return Ok(obj),
            Err(e) => match e {
                sqlx::Error::RowNotFound => $create($db, $tg).await,
                _ => Err(e)
            }
        }
    };

    ($db:expr, $tg:expr, $res:expr, $create: expr $(, $args: expr)*) => {
        match $res {
            Ok(obj) => return Ok(obj),
            Err(e) => match e {
                sqlx::Error::RowNotFound => $create($db, $tg $(,$args)*).await,
                _ => Err(e)
            }
        }
    }
}