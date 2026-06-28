use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::posts::Post;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSubscription {
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub subscribed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadSubscription {
    pub user_id: Uuid,
    pub channel_id: Uuid,
    pub thread_id: Uuid,
    pub subscribed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedPost {
    pub post: Post,
    pub channel_id: Uuid,
    pub thread_id: Uuid,
}
