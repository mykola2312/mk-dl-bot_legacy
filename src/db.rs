use sqlx::migrate::MigrateDatabase;
use sqlx::{Sqlite, SqlitePool};

use super::util::make_database_url;

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
