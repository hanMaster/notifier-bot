use crate::config::config;
use crate::Result;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Sqlite, SqlitePool};

pub mod deal;
pub mod sync;
pub mod deadline;
pub mod stat;

pub struct Db {
    pub db: SqlitePool,
}

impl Db {
    pub async fn new() -> Db {
        let db_result = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&config().DB_URL)
            .await;

        match db_result {
            Ok(db) => Self { db },
            Err(e) => {
                panic!("Create db pool failed: {e}")
            }
        }
    }
}

async fn create_schema(db_url: &str) -> Result<()> {
    let pool = SqlitePool::connect(db_url).await?;
    let qry = r#"
    CREATE TABLE IF NOT EXISTS deal
    (
        id                  INTEGER PRIMARY KEY AUTOINCREMENT,
        deal_id             BIGINTEGER          NOT NULL,
        project             TEXT                NOT NULL,
        house               INTEGER             NOT NULL,
        object_type         TEXT                NOT NULL,
        object              INTEGER             NOT NULL,
        facing              TEXT,
        days_limit          INTEGER  DEFAULT    30,
        transfer_completed  BOOLEAN DEFAULT FALSE,
        created_on          DATETIME DEFAULT    (datetime('now', 'localtime')),
        updated_on          DATETIME DEFAULT    (datetime('now', 'localtime'))
    );
    "#;
    let _ = sqlx::query(qry).execute(&pool).await?;
    pool.close().await;
    Ok(())
}

pub async fn init_db() -> Result<()> {
    if !Sqlite::database_exists(&config().DB_URL)
        .await
        .unwrap_or(false)
    {
        Sqlite::create_database(&config().DB_URL).await?;
        match create_schema(&config().DB_URL).await {
            Ok(_) => log::info!("database created successfully"),
            Err(e) => panic!("{}", e),
        }
    }
    Ok(())
}
