use async_trait::async_trait;
use uuid::Uuid;

use crate::threads::{CreateThread, Thread};
use crate::Result;

#[async_trait]
pub trait ThreadsApi: Send + Sync {
    async fn list_threads(&self, channel_id: Uuid) -> Result<Vec<Thread>>;
    async fn get_thread(&self, id: Uuid) -> Result<Thread>;
    async fn create_thread(&self, input: CreateThread) -> Result<Thread>;
}
