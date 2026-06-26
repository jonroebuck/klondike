use axum::body::Body;
use http::Request;
use tower::ServiceExt;

use klondike_core::artifacts::Artifact;
use klondike_core::channels::Channel;
use klondike_core::issues::{Issue, IssueEvent, IssueStatus};
use klondike_core::posts::Post;
use klondike_core::threads::Thread;

async fn app() -> axum::Router {
    let k = klondike::Klondike::new("sqlite::memory:").await.unwrap();
    k.router()
}

fn json_request(method: &str, uri: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

fn get_request(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .unwrap()
}

async fn body_json<T: serde::de::DeserializeOwned>(response: axum::response::Response) -> T {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn channels_crud() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/channels",
            serde_json::json!({ "name": "general", "description": "General chat" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let channel: Channel = body_json(resp).await;
    assert_eq!(channel.name, "general");

    let resp = app
        .clone()
        .oneshot(get_request(&format!("/api/v1/channels/{}", channel.id)))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let fetched: Channel = body_json(resp).await;
    assert_eq!(fetched.id, channel.id);

    let resp = app
        .clone()
        .oneshot(get_request("/api/v1/channels"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let channels: Vec<Channel> = body_json(resp).await;
    assert_eq!(channels.len(), 1);
    assert_eq!(channels[0].id, channel.id);
}

#[tokio::test]
async fn channel_not_found() {
    let app = app().await;
    let id = uuid::Uuid::new_v4();
    let resp = app
        .oneshot(get_request(&format!("/api/v1/channels/{id}")))
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn threads_in_channel() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/channels",
            serde_json::json!({ "name": "dev", "description": "Dev" }),
        ))
        .await
        .unwrap();
    let channel: Channel = body_json(resp).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/channels/{}/threads", channel.id),
            serde_json::json!({
                "title": "Design review",
                "author": "alice"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let thread: Thread = body_json(resp).await;
    assert_eq!(thread.title, "Design review");
    assert_eq!(thread.channel_id, channel.id);

    let resp = app
        .clone()
        .oneshot(get_request(&format!(
            "/api/v1/channels/{}/threads",
            channel.id
        )))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let threads: Vec<Thread> = body_json(resp).await;
    assert_eq!(threads.len(), 1);
    assert_eq!(threads[0].id, thread.id);

    let resp = app
        .clone()
        .oneshot(get_request(&format!("/api/v1/threads/{}", thread.id)))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let fetched: Thread = body_json(resp).await;
    assert_eq!(fetched.title, "Design review");
}

#[tokio::test]
async fn posts_append_and_order() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/channels",
            serde_json::json!({ "name": "chat", "description": "Chat" }),
        ))
        .await
        .unwrap();
    let channel: Channel = body_json(resp).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &format!("/api/v1/channels/{}/threads", channel.id),
            serde_json::json!({
                "title": "Hello",
                "author": "bob"
            }),
        ))
        .await
        .unwrap();
    let thread: Thread = body_json(resp).await;

    let posts_url = format!("/api/v1/threads/{}/posts", thread.id);

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &posts_url,
            serde_json::json!({ "author": "bob", "content": "First" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let p1: Post = body_json(resp).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &posts_url,
            serde_json::json!({ "author": "alice", "content": "Second" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let p2: Post = body_json(resp).await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            &posts_url,
            serde_json::json!({ "author": "bob", "content": "Third" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let p3: Post = body_json(resp).await;

    let resp = app
        .clone()
        .oneshot(get_request(&posts_url))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let posts: Vec<Post> = body_json(resp).await;
    assert_eq!(posts.len(), 3);
    assert_eq!(posts[0].id, p1.id);
    assert_eq!(posts[0].content, "First");
    assert_eq!(posts[1].id, p2.id);
    assert_eq!(posts[1].content, "Second");
    assert_eq!(posts[2].id, p3.id);
    assert_eq!(posts[2].content, "Third");
}

#[tokio::test]
async fn issue_lifecycle_and_events() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/issues",
            serde_json::json!({
                "title": "Fix login bug",
                "description": "Users can't log in",
                "assignee": "alice"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let issue: Issue = body_json(resp).await;
    assert_eq!(issue.status, IssueStatus::Backlog);
    assert_eq!(issue.assignee, Some("alice".into()));

    let resp = app
        .clone()
        .oneshot(json_request(
            "PATCH",
            &format!("/api/v1/issues/{}/status", issue.id),
            serde_json::json!({ "status": "in_progress", "note": "Starting work" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let issue: Issue = body_json(resp).await;
    assert_eq!(issue.status, IssueStatus::InProgress);

    let resp = app
        .clone()
        .oneshot(json_request(
            "PATCH",
            &format!("/api/v1/issues/{}/status", issue.id),
            serde_json::json!({ "status": "done", "note": "Deployed fix" }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let issue: Issue = body_json(resp).await;
    assert_eq!(issue.status, IssueStatus::Done);

    let resp = app
        .clone()
        .oneshot(get_request(&format!("/api/v1/issues/{}", issue.id)))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let fetched: Issue = body_json(resp).await;
    assert_eq!(fetched.status, IssueStatus::Done);

    let resp = app
        .clone()
        .oneshot(get_request(&format!(
            "/api/v1/issues/{}/events",
            issue.id
        )))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let events: Vec<IssueEvent> = body_json(resp).await;
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].from_status, IssueStatus::Backlog);
    assert_eq!(events[0].to_status, IssueStatus::InProgress);
    assert_eq!(events[0].note, Some("Starting work".into()));
    assert_eq!(events[1].from_status, IssueStatus::InProgress);
    assert_eq!(events[1].to_status, IssueStatus::Done);
    assert_eq!(events[1].note, Some("Deployed fix".into()));

    let resp = app
        .clone()
        .oneshot(get_request("/api/v1/issues"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let issues: Vec<Issue> = body_json(resp).await;
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].id, issue.id);
}

#[tokio::test]
async fn artifact_create_and_read_back() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/artifacts",
            serde_json::json!({
                "name": "design-doc",
                "version": "1.0.0",
                "source_type": "markdown",
                "source_location": "/docs/design.md",
                "content_type": "text/markdown"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let artifact: Artifact = body_json(resp).await;
    assert_eq!(artifact.name, "design-doc");
    assert_eq!(artifact.version, "1.0.0");

    let resp = app
        .clone()
        .oneshot(get_request(&format!("/api/v1/artifacts/{}", artifact.id)))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let fetched: Artifact = body_json(resp).await;
    assert_eq!(fetched.id, artifact.id);
    assert_eq!(fetched.name, "design-doc");
    assert_eq!(fetched.source_location, "/docs/design.md");
}

#[tokio::test]
async fn artifact_list_versioning() {
    let app = app().await;

    for (version, loc) in [("1.0.0", "/v1"), ("2.0.0", "/v2"), ("3.0.0", "/v3")] {
        let resp = app
            .clone()
            .oneshot(json_request(
                "POST",
                "/api/v1/artifacts",
                serde_json::json!({
                    "name": "schema",
                    "version": version,
                    "source_type": "sql",
                    "source_location": loc,
                    "content_type": "text/sql"
                }),
            ))
            .await
            .unwrap();
        assert_eq!(resp.status(), 201);
    }

    let resp = app
        .clone()
        .oneshot(get_request("/api/v1/artifacts"))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let artifacts: Vec<Artifact> = body_json(resp).await;
    assert_eq!(artifacts.len(), 3);
    assert_eq!(artifacts[0].version, "1.0.0");
    assert_eq!(artifacts[1].version, "2.0.0");
    assert_eq!(artifacts[2].version, "3.0.0");
}

#[tokio::test]
async fn artifact_content_roundtrip() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/artifacts",
            serde_json::json!({
                "name": "readme",
                "version": "1.0.0",
                "source_type": "markdown",
                "source_location": "/docs/readme.md",
                "content_type": "text/markdown",
                "content": [72, 101, 108, 108, 111]
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let artifact: Artifact = body_json(resp).await;

    let resp = app
        .clone()
        .oneshot(get_request(&format!(
            "/api/v1/artifacts/{}/content",
            artifact.id
        )))
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers().get("content-type").unwrap().to_str().unwrap(),
        "text/markdown"
    );
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&bytes[..], b"Hello");
}

#[tokio::test]
async fn artifact_content_no_content_returns_204() {
    let app = app().await;

    let resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/v1/artifacts",
            serde_json::json!({
                "name": "empty",
                "version": "1.0.0",
                "source_type": "sql",
                "source_location": "/db/schema.sql",
                "content_type": "text/sql"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let artifact: Artifact = body_json(resp).await;

    let resp = app
        .clone()
        .oneshot(get_request(&format!(
            "/api/v1/artifacts/{}/content",
            artifact.id
        )))
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
}
