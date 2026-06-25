use async_trait::async_trait;
use uuid::Uuid;

use crate::posts::{CreatePost, Post};
use crate::Result;

#[async_trait]
pub trait PostsStorage: Send + Sync {
    async fn list_posts(&self, thread_id: Uuid) -> Result<Vec<Post>>;
    async fn create_post(&self, input: CreatePost) -> Result<Post>;
}
