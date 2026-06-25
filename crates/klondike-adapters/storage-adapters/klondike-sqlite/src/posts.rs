use async_trait::async_trait;
use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use klondike_core::posts::{CreatePost, Post};
use klondike_core::storage::PostsStorage;
use klondike_core::{Error, Result};

use crate::SqliteStorage;

fn row_to_post(row: sqlx::sqlite::SqliteRow) -> std::result::Result<Post, sqlx::Error> {
    Ok(Post {
        id: row.try_get::<String, _>("id")?.parse().map_err(|e: uuid::Error| sqlx::Error::Decode(e.into()))?,
        thread_id: row.try_get::<String, _>("thread_id")?.parse().map_err(|e: uuid::Error| sqlx::Error::Decode(e.into()))?,
        author: row.try_get("author")?,
        content: row.try_get("content")?,
        created_at: row.try_get::<String, _>("created_at")?.parse().map_err(|e: chrono::ParseError| sqlx::Error::Decode(e.into()))?,
    })
}

#[async_trait]
impl PostsStorage for SqliteStorage {
    async fn list_posts(&self, thread_id: Uuid) -> Result<Vec<Post>> {
        let rows = sqlx::query("SELECT id, thread_id, author, content, created_at FROM posts WHERE thread_id = ? ORDER BY created_at")
            .bind(thread_id.to_string())
            .fetch_all(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        rows.into_iter()
            .map(|r| row_to_post(r).map_err(|e| Error::Storage(e.to_string())))
            .collect()
    }

    async fn create_post(&self, input: CreatePost) -> Result<Post> {
        let post = Post {
            id: Uuid::new_v4(),
            thread_id: input.thread_id,
            author: input.author,
            content: input.content,
            created_at: Utc::now(),
        };

        sqlx::query("INSERT INTO posts (id, thread_id, author, content, created_at) VALUES (?, ?, ?, ?, ?)")
            .bind(post.id.to_string())
            .bind(post.thread_id.to_string())
            .bind(&post.author)
            .bind(&post.content)
            .bind(post.created_at.to_rfc3339())
            .execute(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        Ok(post)
    }
}
