use klondike::Klondike;
use klondike_core::api::*;
use klondike_core::artifacts::CreateArtifact;
use klondike_core::channels::CreateChannel;
use klondike_core::issues::{CreateIssue, IssueStatus, UpdateIssueStatus};
use klondike_core::posts::CreatePost;
use klondike_core::threads::CreateThread;

async fn setup() -> Klondike {
    Klondike::new("sqlite::memory:").await.unwrap()
}

#[tokio::test]
async fn channel_create_get_list() {
    let k = setup().await;

    let created = k
        .create_channel(CreateChannel {
            name: "general".into(),
            description: "General discussion".into(),
        })
        .await
        .unwrap();

    assert_eq!(created.name, "general");
    assert_eq!(created.description, "General discussion");

    let fetched = k.get_channel(created.id).await.unwrap();
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "general");

    let channels = k.list_channels().await.unwrap();
    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0].id, created.id);
}

#[tokio::test]
async fn thread_in_channel() {
    let k = setup().await;

    let channel = k
        .create_channel(CreateChannel {
            name: "dev".into(),
            description: "Dev chat".into(),
        })
        .await
        .unwrap();

    let thread = k
        .create_thread(channel.id, CreateThread {
            title: "RFC: new API".into(),
            author: "alice".into(),
        })
        .await
        .unwrap();

    assert_eq!(thread.channel_id, channel.id);
    assert_eq!(thread.title, "RFC: new API");
    assert_eq!(thread.author, "alice");

    let threads = k.list_threads(channel.id).await.unwrap();
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0].id, thread.id);

    let fetched = k.get_thread(thread.id).await.unwrap();
    assert_eq!(fetched.title, "RFC: new API");
}

#[tokio::test]
async fn posts_append_and_order() {
    let k = setup().await;

    let channel = k
        .create_channel(CreateChannel {
            name: "chat".into(),
            description: "Chat".into(),
        })
        .await
        .unwrap();

    let thread = k
        .create_thread(channel.id, CreateThread {
            title: "Hello".into(),
            author: "bob".into(),
        })
        .await
        .unwrap();

    let p1 = k
        .create_post(thread.id, CreatePost {
            author: "bob".into(),
            content: "First post".into(),
        })
        .await
        .unwrap();

    let p2 = k
        .create_post(thread.id, CreatePost {
            author: "alice".into(),
            content: "Second post".into(),
        })
        .await
        .unwrap();

    let p3 = k
        .create_post(thread.id, CreatePost {
            author: "bob".into(),
            content: "Third post".into(),
        })
        .await
        .unwrap();

    let posts = k.list_posts(thread.id).await.unwrap();
    assert_eq!(posts.len(), 3);
    assert_eq!(posts[0].id, p1.id);
    assert_eq!(posts[1].id, p2.id);
    assert_eq!(posts[2].id, p3.id);
    assert_eq!(posts[0].content, "First post");
    assert_eq!(posts[1].content, "Second post");
    assert_eq!(posts[2].content, "Third post");
}

#[tokio::test]
async fn issue_status_transitions_and_events() {
    let k = setup().await;

    let issue = k
        .create_issue(CreateIssue {
            title: "Fix bug".into(),
            description: "Something is broken".into(),
            assignee: Some("alice".into()),
        })
        .await
        .unwrap();

    assert_eq!(issue.status, IssueStatus::Backlog);
    assert_eq!(issue.assignee, Some("alice".into()));

    let issue = k
        .update_status(
            issue.id,
            UpdateIssueStatus {
                status: IssueStatus::InProgress,
                note: Some("Starting work".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(issue.status, IssueStatus::InProgress);

    let issue = k
        .update_status(
            issue.id,
            UpdateIssueStatus {
                status: IssueStatus::Done,
                note: Some("Fixed in commit abc123".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(issue.status, IssueStatus::Done);

    let events = k.list_events(issue.id).await.unwrap();
    assert_eq!(events.len(), 2);

    assert_eq!(events[0].from_status, IssueStatus::Backlog);
    assert_eq!(events[0].to_status, IssueStatus::InProgress);
    assert_eq!(events[0].note, Some("Starting work".into()));

    assert_eq!(events[1].from_status, IssueStatus::InProgress);
    assert_eq!(events[1].to_status, IssueStatus::Done);
    assert_eq!(events[1].note, Some("Fixed in commit abc123".into()));

    let fetched = k.get_issue(issue.id).await.unwrap();
    assert_eq!(fetched.status, IssueStatus::Done);
}

#[tokio::test]
async fn artifact_create_and_read_back() {
    let k = setup().await;

    let artifact = k
        .create_artifact(CreateArtifact {
            name: "design-doc".into(),
            version: "1.0.0".into(),
            source_type: "markdown".into(),
            source_location: "/docs/design.md".into(),
            content_type: "text/markdown".into(),
            content: Some(b"# Design Doc\nHello world".to_vec()),
        })
        .await
        .unwrap();

    assert_eq!(artifact.name, "design-doc");
    assert_eq!(artifact.version, "1.0.0");
    assert_eq!(artifact.source_type, "markdown");
    assert_eq!(artifact.source_location, "/docs/design.md");
    assert_eq!(artifact.content_type, "text/markdown");

    let fetched = k.get_artifact(artifact.id).await.unwrap();
    assert_eq!(fetched.id, artifact.id);
    assert_eq!(fetched.name, "design-doc");
    assert_eq!(fetched.version, "1.0.0");

    let content = k.get_artifact_content(artifact.id).await.unwrap();
    assert_eq!(content, Some(b"# Design Doc\nHello world".to_vec()));
}

#[tokio::test]
async fn artifact_list_and_versioning() {
    let k = setup().await;

    let v1 = k
        .create_artifact(CreateArtifact {
            name: "schema".into(),
            version: "1.0.0".into(),
            source_type: "sql".into(),
            source_location: "/db/schema.sql".into(),
            content_type: "text/sql".into(),
            content: None,
        })
        .await
        .unwrap();

    let v2 = k
        .create_artifact(CreateArtifact {
            name: "schema".into(),
            version: "2.0.0".into(),
            source_type: "sql".into(),
            source_location: "/db/schema_v2.sql".into(),
            content_type: "text/sql".into(),
            content: None,
        })
        .await
        .unwrap();

    let v3 = k
        .create_artifact(CreateArtifact {
            name: "schema".into(),
            version: "3.0.0".into(),
            source_type: "sql".into(),
            source_location: "/db/schema_v3.sql".into(),
            content_type: "text/sql".into(),
            content: None,
        })
        .await
        .unwrap();

    let all = k.list_artifacts().await.unwrap();
    assert_eq!(all.len(), 3);

    assert_eq!(all[0].id, v1.id);
    assert_eq!(all[0].version, "1.0.0");
    assert_eq!(all[1].id, v2.id);
    assert_eq!(all[1].version, "2.0.0");
    assert_eq!(all[2].id, v3.id);
    assert_eq!(all[2].version, "3.0.0");

    let content = k.get_artifact_content(v1.id).await.unwrap();
    assert_eq!(content, None);
}

#[tokio::test]
async fn get_nonexistent_channel_returns_not_found() {
    let k = setup().await;
    let result = k.get_channel(uuid::Uuid::new_v4()).await;
    assert!(result.is_err());
}
