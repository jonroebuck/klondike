use async_trait::async_trait;
use uuid::Uuid;

use crate::issues::{CreateIssue, Issue, IssueEvent, UpdateIssueStatus};
use crate::Result;

#[async_trait]
pub trait IssuesApi: Send + Sync {
    async fn list_issues(&self) -> Result<Vec<Issue>>;
    async fn get_issue(&self, id: Uuid) -> Result<Issue>;
    async fn create_issue(&self, input: CreateIssue) -> Result<Issue>;
    async fn update_status(&self, id: Uuid, input: UpdateIssueStatus) -> Result<Issue>;
    async fn list_events(&self, issue_id: Uuid) -> Result<Vec<IssueEvent>>;
}
