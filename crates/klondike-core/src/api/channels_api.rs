use async_trait::async_trait;
use uuid::Uuid;

use crate::channels::{Channel, CreateChannel};
use crate::Result;

#[async_trait]
pub trait ChannelsApi: Send + Sync {
    async fn list_channels(&self) -> Result<Vec<Channel>>;
    async fn get_channel(&self, id: Uuid) -> Result<Channel>;
    async fn create_channel(&self, input: CreateChannel) -> Result<Channel>;
}
