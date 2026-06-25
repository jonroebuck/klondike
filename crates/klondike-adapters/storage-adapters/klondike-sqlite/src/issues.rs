use async_trait::async_trait;
use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use klondike_core::issues::{CreateIssue, Issue, IssueEvent, IssueStatus};
use klondike_core::storage::IssuesStorage;
use klondike_core::{Error, Result};

use crate::SqliteStorage;

fn row_to_issue(row: sqlx::sqlite::SqliteRow) -> std::result::Result<Issue, Error> {
    Ok(Issue {
        id: row.try_get::<String, _>("id").map_err(|e| Error::Storage(e.to_string()))?.parse().map_err(|e: uuid::Error| Error::Storage(e.to_string()))?,
        title: row.try_get("title").map_err(|e| Error::Storage(e.to_string()))?,
        description: row.try_get("description").map_err(|e| Error::Storage(e.to_string()))?,
        status: row.try_get::<String, _>("status").map_err(|e| Error::Storage(e.to_string()))?.parse()?,
        assignee: row.try_get("assignee").map_err(|e| Error::Storage(e.to_string()))?,
        created_at: row.try_get::<String, _>("created_at").map_err(|e| Error::Storage(e.to_string()))?.parse().map_err(|e: chrono::ParseError| Error::Storage(e.to_string()))?,
        updated_at: row.try_get::<String, _>("updated_at").map_err(|e| Error::Storage(e.to_string()))?.parse().map_err(|e: chrono::ParseError| Error::Storage(e.to_string()))?,
    })
}

fn row_to_event(row: sqlx::sqlite::SqliteRow) -> std::result::Result<IssueEvent, Error> {
    Ok(IssueEvent {
        id: row.try_get::<String, _>("id").map_err(|e| Error::Storage(e.to_string()))?.parse().map_err(|e: uuid::Error| Error::Storage(e.to_string()))?,
        issue_id: row.try_get::<String, _>("issue_id").map_err(|e| Error::Storage(e.to_string()))?.parse().map_err(|e: uuid::Error| Error::Storage(e.to_string()))?,
        from_status: row.try_get::<String, _>("from_status").map_err(|e| Error::Storage(e.to_string()))?.parse()?,
        to_status: row.try_get::<String, _>("to_status").map_err(|e| Error::Storage(e.to_string()))?.parse()?,
        note: row.try_get("note").map_err(|e| Error::Storage(e.to_string()))?,
        timestamp: row.try_get::<String, _>("timestamp").map_err(|e| Error::Storage(e.to_string()))?.parse().map_err(|e: chrono::ParseError| Error::Storage(e.to_string()))?,
    })
}

#[async_trait]
impl IssuesStorage for SqliteStorage {
    async fn list_issues(&self) -> Result<Vec<Issue>> {
        let rows = sqlx::query("SELECT id, title, description, status, assignee, created_at, updated_at FROM issues ORDER BY created_at")
            .fetch_all(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        rows.into_iter().map(row_to_issue).collect()
    }

    async fn get_issue(&self, id: Uuid) -> Result<Issue> {
        let row = sqlx::query("SELECT id, title, description, status, assignee, created_at, updated_at FROM issues WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?
            .ok_or(Error::NotFound { resource: "issue", id })?;

        row_to_issue(row)
    }

    async fn create_issue(&self, input: CreateIssue) -> Result<Issue> {
        let now = Utc::now();
        let issue = Issue {
            id: Uuid::new_v4(),
            title: input.title,
            description: input.description,
            status: IssueStatus::Backlog,
            assignee: input.assignee,
            created_at: now,
            updated_at: now,
        };

        sqlx::query("INSERT INTO issues (id, title, description, status, assignee, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind(issue.id.to_string())
            .bind(&issue.title)
            .bind(&issue.description)
            .bind(issue.status.to_string())
            .bind(&issue.assignee)
            .bind(issue.created_at.to_rfc3339())
            .bind(issue.updated_at.to_rfc3339())
            .execute(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        Ok(issue)
    }

    async fn update_status(
        &self,
        id: Uuid,
        from: IssueStatus,
        to: IssueStatus,
        note: Option<String>,
    ) -> Result<Issue> {
        let now = Utc::now();

        let result = sqlx::query("UPDATE issues SET status = ?, updated_at = ? WHERE id = ? AND status = ?")
            .bind(to.to_string())
            .bind(now.to_rfc3339())
            .bind(id.to_string())
            .bind(from.to_string())
            .execute(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound { resource: "issue", id });
        }

        let event_id = Uuid::new_v4();
        sqlx::query("INSERT INTO issue_events (id, issue_id, from_status, to_status, note, timestamp) VALUES (?, ?, ?, ?, ?, ?)")
            .bind(event_id.to_string())
            .bind(id.to_string())
            .bind(from.to_string())
            .bind(to.to_string())
            .bind(&note)
            .bind(now.to_rfc3339())
            .execute(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        self.get_issue(id).await
    }

    async fn list_events(&self, issue_id: Uuid) -> Result<Vec<IssueEvent>> {
        let rows = sqlx::query("SELECT id, issue_id, from_status, to_status, note, timestamp FROM issue_events WHERE issue_id = ? ORDER BY timestamp")
            .bind(issue_id.to_string())
            .fetch_all(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        rows.into_iter().map(row_to_event).collect()
    }
}
