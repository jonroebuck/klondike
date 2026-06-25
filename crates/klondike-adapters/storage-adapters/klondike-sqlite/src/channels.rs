use async_trait::async_trait;
use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use klondike_core::channels::{Channel, CreateChannel};
use klondike_core::storage::ChannelsStorage;
use klondike_core::{Error, Result};

use crate::SqliteStorage;

fn row_to_channel(row: sqlx::sqlite::SqliteRow) -> std::result::Result<Channel, sqlx::Error> {
    Ok(Channel {
        id: row.try_get::<String, _>("id")?.parse().map_err(|e: uuid::Error| sqlx::Error::Decode(e.into()))?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        created_at: row.try_get::<String, _>("created_at")?.parse().map_err(|e: chrono::ParseError| sqlx::Error::Decode(e.into()))?,
    })
}

#[async_trait]
impl ChannelsStorage for SqliteStorage {
    async fn list_channels(&self) -> Result<Vec<Channel>> {
        let rows = sqlx::query("SELECT id, name, description, created_at FROM channels ORDER BY created_at")
            .fetch_all(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        rows.into_iter()
            .map(|r| row_to_channel(r).map_err(|e| Error::Storage(e.to_string())))
            .collect()
    }

    async fn get_channel(&self, id: Uuid) -> Result<Channel> {
        let row = sqlx::query("SELECT id, name, description, created_at FROM channels WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?
            .ok_or(Error::NotFound { resource: "channel", id })?;

        row_to_channel(row).map_err(|e| Error::Storage(e.to_string()))
    }

    async fn create_channel(&self, input: CreateChannel) -> Result<Channel> {
        let channel = Channel {
            id: Uuid::new_v4(),
            name: input.name,
            description: input.description,
            created_at: Utc::now(),
        };

        sqlx::query("INSERT INTO channels (id, name, description, created_at) VALUES (?, ?, ?, ?)")
            .bind(channel.id.to_string())
            .bind(&channel.name)
            .bind(&channel.description)
            .bind(channel.created_at.to_rfc3339())
            .execute(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        Ok(channel)
    }
}
