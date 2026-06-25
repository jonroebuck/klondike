mod channels;
mod threads;
mod posts;
mod issues;
mod artifacts;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

const MIGRATIONS_001: &str = include_str!("../migrations/001_init.sql");
const MIGRATIONS_002: &str = include_str!("../migrations/002_artifact_content.sql");

#[derive(Clone)]
pub struct SqliteStorage {
    pool: SqlitePool,
}

impl SqliteStorage {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let max_conn = if database_url.contains(":memory:") { 1 } else { 5 };
        let pool = SqlitePoolOptions::new()
            .max_connections(max_conn)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::raw_sql(MIGRATIONS_001).execute(&self.pool).await?;
        sqlx::raw_sql(MIGRATIONS_002).execute(&self.pool).await?;
        Ok(())
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
