use sqlx::migrate::MigrateDatabase;
use sqlx::{PgPool, Postgres};
use std::fmt;

use super::util::unwrap_env;

pub type DbPool = PgPool;

#[derive(sqlx::FromRow, Debug)]
pub struct User {
    pub id: i32,
    pub tg_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub can_download: bool,
    pub is_admin: bool,
    pub has_private_chat: bool,
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

#[derive(sqlx::FromRow, Debug)]
pub struct Chat {
    pub id: i32,
    pub tg_id: i64,
    pub username: Option<String>,
    pub title: String,
    pub can_download: bool,
}

impl fmt::Display for Chat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} - {}", self.tg_id, self.username_or_title())
    }
}

impl Chat {
    pub fn username_or_title(&self) -> &String {
        self.username.as_ref().unwrap_or(&self.title)
    }
}

pub mod chat;

#[derive(sqlx::FromRow, Debug)]
pub struct Link {
    pub id: i32,
    pub domain: String,
    pub path: Option<String>,
    pub download_allowed: bool,
    pub auto_download: bool,
}

#[derive(sqlx::FromRow, Debug)]
pub struct Request {
    pub id: i32,
    pub requested_by: i32,
    pub approved_by: Option<i32>,
    pub message: String,
    pub is_approved: bool,
}

#[derive(sqlx::FromRow, Debug)]
pub struct RequestChat {
    pub id: i32,
    pub requested_by: i32,
    pub requested_for: i32,
    pub approved_by: Option<i32>,
    pub message: String,
    pub is_approved: bool,
}

pub fn make_database_url() -> String {
    format!(
        "postgres://{}:{}@{}/{}",
        unwrap_env("POSTGRES_USER"),
        unwrap_env("POSTGRES_PASSWORD"),
        unwrap_env("POSTGRES_HOST"),
        unwrap_env("POSTGRES_DB")
    )
}

pub async fn db_init() -> PgPool {
    let db_url = make_database_url();
    if !Postgres::database_exists(&db_url).await.unwrap_or(false) {
        Postgres::create_database(&db_url)
            .await
            .expect("failed to create database");
    }

    let db = PgPool::connect(&db_url).await.unwrap();
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
