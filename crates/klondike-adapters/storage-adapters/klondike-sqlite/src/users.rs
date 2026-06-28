use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use klondike_core::posts::Post;
use klondike_core::storage::UserStorage;
use klondike_core::users::{ChannelSubscription, FeedPost, ThreadSubscription, User};
use klondike_core::{Error, Result};

use crate::SqliteStorage;

// ── Row decoding helpers ──────────────────────────────────────────────────────

fn col_str(row: &sqlx::sqlite::SqliteRow, col: &str) -> Result<String> {
    row.try_get::<String, _>(col)
        .map_err(|e| Error::Storage(e.to_string()))
}

fn col_uuid(row: &sqlx::sqlite::SqliteRow, col: &str) -> Result<Uuid> {
    col_str(row, col)?
        .parse::<Uuid>()
        .map_err(|e| Error::Storage(e.to_string()))
}

fn col_datetime(row: &sqlx::sqlite::SqliteRow, col: &str) -> Result<DateTime<Utc>> {
    col_str(row, col)?
        .parse::<DateTime<Utc>>()
        .map_err(|e| Error::Storage(e.to_string()))
}

fn row_to_post(row: &sqlx::sqlite::SqliteRow) -> Result<Post> {
    Ok(Post {
        id: col_uuid(row, "id")?,
        thread_id: col_uuid(row, "thread_id")?,
        author: col_str(row, "author")?,
        content: col_str(row, "content")?,
        created_at: col_datetime(row, "created_at")?,
    })
}

fn row_to_channel_sub(row: &sqlx::sqlite::SqliteRow) -> Result<ChannelSubscription> {
    Ok(ChannelSubscription {
        user_id: col_uuid(row, "user_id")?,
        channel_id: col_uuid(row, "channel_id")?,
        subscribed_at: col_datetime(row, "subscribed_at")?,
    })
}

fn row_to_thread_sub(row: &sqlx::sqlite::SqliteRow) -> Result<ThreadSubscription> {
    Ok(ThreadSubscription {
        user_id: col_uuid(row, "user_id")?,
        thread_id: col_uuid(row, "thread_id")?,
        channel_id: col_uuid(row, "channel_id")?,
        subscribed_at: col_datetime(row, "subscribed_at")?,
    })
}

// ── Guard helper ──────────────────────────────────────────────────────────────

async fn require_user(pool: &sqlx::SqlitePool, user_id: Uuid) -> Result<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE id = ?")
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| Error::Storage(e.to_string()))?;
    if count == 0 {
        Err(Error::NotFound { resource: "user", id: user_id })
    } else {
        Ok(())
    }
}

// ── Trait implementation ──────────────────────────────────────────────────────

#[async_trait]
impl UserStorage for SqliteStorage {
    async fn register_user(&self) -> Result<User> {
        let user = User { id: Uuid::new_v4(), created_at: Utc::now() };
        sqlx::query("INSERT INTO users (id, created_at) VALUES (?, ?)")
            .bind(user.id.to_string())
            .bind(user.created_at.to_rfc3339())
            .execute(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;
        Ok(user)
    }

    async fn unregister_user(&self, user_id: Uuid) -> Result<()> {
        require_user(self.pool(), user_id).await?;
        let id = user_id.to_string();
        sqlx::query("DELETE FROM thread_subscriptions WHERE user_id = ?")
            .bind(&id)
            .execute(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;
        sqlx::query("DELETE FROM channel_subscriptions WHERE user_id = ?")
            .bind(&id)
            .execute(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;
        sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(&id)
            .execute(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;
        Ok(())
    }

    async fn subscribe_to_channel(
        &self,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> Result<ChannelSubscription> {
        require_user(self.pool(), user_id).await?;
        let now = Utc::now();
        sqlx::query(
            "INSERT OR IGNORE INTO channel_subscriptions (user_id, channel_id, subscribed_at)
             VALUES (?, ?, ?)",
        )
        .bind(user_id.to_string())
        .bind(channel_id.to_string())
        .bind(now.to_rfc3339())
        .execute(self.pool())
        .await
        .map_err(|e| Error::Storage(e.to_string()))?;

        let row = sqlx::query(
            "SELECT user_id, channel_id, subscribed_at
             FROM channel_subscriptions WHERE user_id = ? AND channel_id = ?",
        )
        .bind(user_id.to_string())
        .bind(channel_id.to_string())
        .fetch_optional(self.pool())
        .await
        .map_err(|e| Error::Storage(e.to_string()))?
        .ok_or(Error::NotFound { resource: "channel", id: channel_id })?;

        row_to_channel_sub(&row)
    }

    async fn unsubscribe_from_channel(
        &self,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM channel_subscriptions WHERE user_id = ? AND channel_id = ?",
        )
        .bind(user_id.to_string())
        .bind(channel_id.to_string())
        .execute(self.pool())
        .await
        .map_err(|e| Error::Storage(e.to_string()))?;
        Ok(())
    }

    async fn subscribe_to_thread(
        &self,
        user_id: Uuid,
        channel_id: Uuid,
        thread_id: Uuid,
    ) -> Result<ThreadSubscription> {
        require_user(self.pool(), user_id).await?;
        let now = Utc::now();
        sqlx::query(
            "INSERT OR IGNORE INTO thread_subscriptions
             (user_id, thread_id, channel_id, subscribed_at)
             VALUES (?, ?, ?, ?)",
        )
        .bind(user_id.to_string())
        .bind(thread_id.to_string())
        .bind(channel_id.to_string())
        .bind(now.to_rfc3339())
        .execute(self.pool())
        .await
        .map_err(|e| Error::Storage(e.to_string()))?;

        let row = sqlx::query(
            "SELECT user_id, thread_id, channel_id, subscribed_at
             FROM thread_subscriptions WHERE user_id = ? AND thread_id = ?",
        )
        .bind(user_id.to_string())
        .bind(thread_id.to_string())
        .fetch_optional(self.pool())
        .await
        .map_err(|e| Error::Storage(e.to_string()))?
        .ok_or(Error::NotFound { resource: "thread", id: thread_id })?;

        row_to_thread_sub(&row)
    }

    async fn unsubscribe_from_thread(
        &self,
        user_id: Uuid,
        thread_id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            "DELETE FROM thread_subscriptions WHERE user_id = ? AND thread_id = ?",
        )
        .bind(user_id.to_string())
        .bind(thread_id.to_string())
        .execute(self.pool())
        .await
        .map_err(|e| Error::Storage(e.to_string()))?;
        Ok(())
    }

    async fn get_feed(&self, user_id: Uuid) -> Result<Vec<FeedPost>> {
        require_user(self.pool(), user_id).await?;

        let sub_rows = sqlx::query(
            "SELECT thread_id, channel_id, last_read_post_id
             FROM thread_subscriptions WHERE user_id = ?",
        )
        .bind(user_id.to_string())
        .fetch_all(self.pool())
        .await
        .map_err(|e| Error::Storage(e.to_string()))?;

        let mut feed: Vec<FeedPost> = Vec::new();

        for sub in &sub_rows {
            let thread_id = col_uuid(sub, "thread_id")?;
            let channel_id = col_uuid(sub, "channel_id")?;
            let last_read: Option<String> = sub
                .try_get("last_read_post_id")
                .map_err(|e| Error::Storage(e.to_string()))?;

            // LEFT JOIN with the last-read post gives its created_at in a single
            // query, handling both the NULL case (fetch all) and non-NULL case
            // (fetch only posts newer by created_at).
            let post_rows = sqlx::query(
                "SELECT p.id, p.thread_id, p.author, p.content, p.created_at
                 FROM posts p
                 LEFT JOIN posts lp ON lp.id = ?
                 WHERE p.thread_id = ?
                 AND (lp.id IS NULL OR p.created_at > lp.created_at)
                 ORDER BY p.created_at",
            )
            .bind(&last_read)
            .bind(thread_id.to_string())
            .fetch_all(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

            if let Some(last_row) = post_rows.last() {
                let new_last_id = col_str(last_row, "id")?;
                sqlx::query(
                    "UPDATE thread_subscriptions SET last_read_post_id = ?
                     WHERE user_id = ? AND thread_id = ?",
                )
                .bind(&new_last_id)
                .bind(user_id.to_string())
                .bind(thread_id.to_string())
                .execute(self.pool())
                .await
                .map_err(|e| Error::Storage(e.to_string()))?;
            }

            for row in &post_rows {
                feed.push(FeedPost {
                    post: row_to_post(row)?,
                    channel_id,
                    thread_id,
                });
            }
        }

        Ok(feed)
    }

    async fn list_subscriptions(
        &self,
        user_id: Uuid,
    ) -> Result<(Vec<ChannelSubscription>, Vec<ThreadSubscription>)> {
        require_user(self.pool(), user_id).await?;
        let uid = user_id.to_string();

        let ch_rows = sqlx::query(
            "SELECT user_id, channel_id, subscribed_at
             FROM channel_subscriptions WHERE user_id = ?",
        )
        .bind(&uid)
        .fetch_all(self.pool())
        .await
        .map_err(|e| Error::Storage(e.to_string()))?;

        let channels = ch_rows
            .iter()
            .map(row_to_channel_sub)
            .collect::<Result<Vec<_>>>()?;

        let th_rows = sqlx::query(
            "SELECT user_id, thread_id, channel_id, subscribed_at
             FROM thread_subscriptions WHERE user_id = ?",
        )
        .bind(&uid)
        .fetch_all(self.pool())
        .await
        .map_err(|e| Error::Storage(e.to_string()))?;

        let threads = th_rows
            .iter()
            .map(row_to_thread_sub)
            .collect::<Result<Vec<_>>>()?;

        Ok((channels, threads))
    }
}
