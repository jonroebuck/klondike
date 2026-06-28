use async_trait::async_trait;
use uuid::Uuid;

use crate::users::{ChannelSubscription, FeedPost, ThreadSubscription, User};
use crate::Result;

#[async_trait]
pub trait UserStorage: Send + Sync {
    async fn register_user(&self) -> Result<User>;
    async fn unregister_user(&self, user_id: Uuid) -> Result<()>;

    async fn subscribe_to_channel(
        &self,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> Result<ChannelSubscription>;

    async fn unsubscribe_from_channel(
        &self,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> Result<()>;

    async fn subscribe_to_thread(
        &self,
        user_id: Uuid,
        channel_id: Uuid,
        thread_id: Uuid,
    ) -> Result<ThreadSubscription>;

    async fn unsubscribe_from_thread(
        &self,
        user_id: Uuid,
        thread_id: Uuid,
    ) -> Result<()>;

    async fn get_feed(&self, user_id: Uuid) -> Result<Vec<FeedPost>>;

    async fn list_subscriptions(
        &self,
        user_id: Uuid,
    ) -> Result<(Vec<ChannelSubscription>, Vec<ThreadSubscription>)>;
}
