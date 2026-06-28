use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use klondike_core::api::*;
use klondike_core::artifacts::{Artifact, CreateArtifact};
use klondike_core::channels::{Channel, CreateChannel};
use klondike_core::issues::{CreateIssue, Issue, IssueEvent, UpdateIssueStatus};
use klondike_core::posts::{CreatePost, Post};
use klondike_core::storage::*;
use klondike_core::threads::{CreateThread, Thread};
use klondike_core::users::{ChannelSubscription, FeedPost, ThreadSubscription, User};
use klondike_core::Result;
use klondike_sqlite::SqliteStorage;

pub struct Klondike {
    storage: SqliteStorage,
}

impl Klondike {
    pub async fn new(database_url: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let storage = SqliteStorage::new(database_url).await?;
        storage.migrate().await?;
        Ok(Self { storage })
    }

    pub fn router(self) -> axum::Router {
        let state = Arc::new(self);
        klondike_rest::router(state)
    }
}

#[async_trait]
impl ChannelsApi for Klondike {
    async fn list_channels(&self) -> Result<Vec<Channel>> {
        self.storage.list_channels().await
    }
    async fn get_channel(&self, id: Uuid) -> Result<Channel> {
        self.storage.get_channel(id).await
    }
    async fn create_channel(&self, input: CreateChannel) -> Result<Channel> {
        self.storage.create_channel(input).await
    }
}

#[async_trait]
impl ThreadsApi for Klondike {
    async fn list_threads(&self, channel_id: Uuid) -> Result<Vec<Thread>> {
        self.storage.list_threads(channel_id).await
    }
    async fn get_thread(&self, id: Uuid) -> Result<Thread> {
        self.storage.get_thread(id).await
    }
    async fn create_thread(&self, channel_id: Uuid, input: CreateThread) -> Result<Thread> {
        self.storage.create_thread(channel_id, input).await
    }
}

#[async_trait]
impl PostsApi for Klondike {
    async fn list_posts(&self, thread_id: Uuid) -> Result<Vec<Post>> {
        self.storage.list_posts(thread_id).await
    }
    async fn create_post(&self, thread_id: Uuid, input: CreatePost) -> Result<Post> {
        self.storage.create_post(thread_id, input).await
    }
}

#[async_trait]
impl IssuesApi for Klondike {
    async fn list_issues(&self) -> Result<Vec<Issue>> {
        self.storage.list_issues().await
    }
    async fn get_issue(&self, id: Uuid) -> Result<Issue> {
        self.storage.get_issue(id).await
    }
    async fn create_issue(&self, input: CreateIssue) -> Result<Issue> {
        self.storage.create_issue(input).await
    }
    async fn update_status(&self, id: Uuid, input: UpdateIssueStatus) -> Result<Issue> {
        let issue = self.storage.get_issue(id).await?;
        self.storage
            .update_status(id, issue.status, input.status, input.note)
            .await
    }
    async fn list_events(&self, issue_id: Uuid) -> Result<Vec<IssueEvent>> {
        self.storage.list_events(issue_id).await
    }
}

#[async_trait]
impl ArtifactsApi for Klondike {
    async fn list_artifacts(&self) -> Result<Vec<Artifact>> {
        self.storage.list_artifacts().await
    }
    async fn get_artifact(&self, id: Uuid) -> Result<Artifact> {
        self.storage.get_artifact(id).await
    }
    async fn create_artifact(&self, input: CreateArtifact) -> Result<Artifact> {
        self.storage.create_artifact(input).await
    }
    async fn get_artifact_content(&self, id: Uuid) -> Result<Option<Vec<u8>>> {
        self.storage.get_artifact_content(id).await
    }
}

#[async_trait]
impl UsersApi for Klondike {
    async fn register_user(&self) -> Result<User> {
        self.storage.register_user().await
    }
    async fn unregister_user(&self, user_id: Uuid) -> Result<()> {
        self.storage.unregister_user(user_id).await
    }
    async fn subscribe_to_channel(
        &self,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> Result<ChannelSubscription> {
        self.storage.subscribe_to_channel(user_id, channel_id).await
    }
    async fn unsubscribe_from_channel(
        &self,
        user_id: Uuid,
        channel_id: Uuid,
    ) -> Result<()> {
        self.storage.unsubscribe_from_channel(user_id, channel_id).await
    }
    async fn subscribe_to_thread(
        &self,
        user_id: Uuid,
        channel_id: Uuid,
        thread_id: Uuid,
    ) -> Result<ThreadSubscription> {
        self.storage.subscribe_to_thread(user_id, channel_id, thread_id).await
    }
    async fn unsubscribe_from_thread(
        &self,
        user_id: Uuid,
        thread_id: Uuid,
    ) -> Result<()> {
        self.storage.unsubscribe_from_thread(user_id, thread_id).await
    }
    async fn get_feed(&self, user_id: Uuid) -> Result<Vec<FeedPost>> {
        self.storage.get_feed(user_id).await
    }
    async fn list_subscriptions(
        &self,
        user_id: Uuid,
    ) -> Result<(Vec<ChannelSubscription>, Vec<ThreadSubscription>)> {
        self.storage.list_subscriptions(user_id).await
    }
}
