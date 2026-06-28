mod channels;
mod threads;
mod posts;
mod issues;
mod artifacts;
mod users;

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};

const MIGRATIONS_001: &str = include_str!("../migrations/001_init.sql");
const MIGRATIONS_002: &str = include_str!("../migrations/002_artifact_content.sql");
const MIGRATIONS_003: &str = include_str!("../migrations/003_users.sql");

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
        sqlx::raw_sql(
            "CREATE TABLE IF NOT EXISTS _schema_migrations (
                version INTEGER PRIMARY KEY NOT NULL,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
        )
        .execute(&self.pool)
        .await?;

        // Bootstrap: for databases created before migration tracking was introduced,
        // detect what has already been applied by inspecting the live schema so that
        // re-running migrate() on those databases does not re-execute applied DDL.
        let tracked: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _schema_migrations")
            .fetch_one(&self.pool)
            .await?;

        if tracked == 0 {
            let has_schema: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='channels'",
            )
            .fetch_one(&self.pool)
            .await?;

            if has_schema > 0 {
                sqlx::raw_sql("INSERT INTO _schema_migrations (version) VALUES (1)")
                    .execute(&self.pool)
                    .await?;

                let has_content: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM pragma_table_info('artifacts') WHERE name='content'",
                )
                .fetch_one(&self.pool)
                .await?;

                if has_content > 0 {
                    sqlx::raw_sql("INSERT INTO _schema_migrations (version) VALUES (2)")
                        .execute(&self.pool)
                        .await?;
                }

                let has_users: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'",
                )
                .fetch_one(&self.pool)
                .await?;

                if has_users > 0 {
                    sqlx::raw_sql("INSERT INTO _schema_migrations (version) VALUES (3)")
                        .execute(&self.pool)
                        .await?;
                }
            }
        }

        self.apply_migration(1, MIGRATIONS_001).await?;
        self.apply_migration(2, MIGRATIONS_002).await?;
        self.apply_migration(3, MIGRATIONS_003).await?;
        Ok(())
    }

    async fn apply_migration(&self, version: i64, sql: &str) -> Result<(), sqlx::Error> {
        let applied: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _schema_migrations WHERE version = ?")
                .bind(version)
                .fetch_one(&self.pool)
                .await?;

        if applied == 0 {
            sqlx::raw_sql(sql).execute(&self.pool).await?;
            sqlx::query("INSERT INTO _schema_migrations (version) VALUES (?)")
                .bind(version)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn migrate_is_idempotent() {
        let storage = SqliteStorage::new("sqlite::memory:").await.unwrap();
        storage.migrate().await.unwrap();
        storage.migrate().await.unwrap(); // second call must not fail
    }
}
