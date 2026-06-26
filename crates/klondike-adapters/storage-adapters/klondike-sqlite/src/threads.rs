use async_trait::async_trait;
use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use klondike_core::threads::{CreateThread, Thread};
use klondike_core::storage::ThreadsStorage;
use klondike_core::{Error, Result};

use crate::SqliteStorage;

fn row_to_thread(row: sqlx::sqlite::SqliteRow) -> std::result::Result<Thread, sqlx::Error> {
    Ok(Thread {
        id: row.try_get::<String, _>("id")?.parse().map_err(|e: uuid::Error| sqlx::Error::Decode(e.into()))?,
        channel_id: row.try_get::<String, _>("channel_id")?.parse().map_err(|e: uuid::Error| sqlx::Error::Decode(e.into()))?,
        title: row.try_get("title")?,
        author: row.try_get("author")?,
        created_at: row.try_get::<String, _>("created_at")?.parse().map_err(|e: chrono::ParseError| sqlx::Error::Decode(e.into()))?,
    })
}

#[async_trait]
impl ThreadsStorage for SqliteStorage {
    async fn list_threads(&self, channel_id: Uuid) -> Result<Vec<Thread>> {
        let rows = sqlx::query("SELECT id, channel_id, title, author, created_at FROM threads WHERE channel_id = ? ORDER BY created_at")
            .bind(channel_id.to_string())
            .fetch_all(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        rows.into_iter()
            .map(|r| row_to_thread(r).map_err(|e| Error::Storage(e.to_string())))
            .collect()
    }

    async fn get_thread(&self, id: Uuid) -> Result<Thread> {
        let row = sqlx::query("SELECT id, channel_id, title, author, created_at FROM threads WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?
            .ok_or(Error::NotFound { resource: "thread", id })?;

        row_to_thread(row).map_err(|e| Error::Storage(e.to_string()))
    }

    async fn create_thread(&self, channel_id: Uuid, input: CreateThread) -> Result<Thread> {
        let thread = Thread {
            id: Uuid::new_v4(),
            channel_id,
            title: input.title,
            author: input.author,
            created_at: Utc::now(),
        };

        sqlx::query("INSERT INTO threads (id, channel_id, title, author, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind(thread.id.to_string())
            .bind(thread.channel_id.to_string())
            .bind(&thread.title)
            .bind(&thread.author)
            .bind(thread.created_at.to_rfc3339())
            .execute(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        Ok(thread)
    }
}
