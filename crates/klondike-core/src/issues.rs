use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Backlog,
    InProgress,
    Done,
    Blocked,
}

impl std::fmt::Display for IssueStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Backlog => write!(f, "backlog"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Done => write!(f, "done"),
            Self::Blocked => write!(f, "blocked"),
        }
    }
}

impl std::str::FromStr for IssueStatus {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "backlog" => Ok(Self::Backlog),
            "in_progress" => Ok(Self::InProgress),
            "done" => Ok(Self::Done),
            "blocked" => Ok(Self::Blocked),
            other => Err(crate::Error::InvalidInput(format!(
                "invalid issue status: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub status: IssueStatus,
    pub assignee: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateIssue {
    pub title: String,
    pub description: String,
    pub assignee: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateIssueStatus {
    pub status: IssueStatus,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueEvent {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub from_status: IssueStatus,
    pub to_status: IssueStatus,
    pub note: Option<String>,
    pub timestamp: DateTime<Utc>,
}
