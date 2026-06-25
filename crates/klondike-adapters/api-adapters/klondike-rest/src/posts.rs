use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use klondike_core::api::PostsApi;
use klondike_core::posts::{CreatePost, Post};

use super::error_to_response;

pub fn routes<S: PostsApi + 'static>(state: Arc<S>) -> Router {
    Router::new()
        .route("/api/v1/threads/{thread_id}/posts", get(list).post(create))
        .with_state(state)
}

async fn list<S: PostsApi>(
    State(state): State<Arc<S>>,
    Path(thread_id): Path<Uuid>,
) -> Result<Json<Vec<Post>>, axum::http::StatusCode> {
    state.list_posts(thread_id).await.map(Json).map_err(error_to_response)
}

async fn create<S: PostsApi>(
    State(state): State<Arc<S>>,
    Path(thread_id): Path<Uuid>,
    Json(mut input): Json<CreatePost>,
) -> Result<(axum::http::StatusCode, Json<Post>), axum::http::StatusCode> {
    input.thread_id = thread_id;
    state
        .create_post(input)
        .await
        .map(|p| (axum::http::StatusCode::CREATED, Json(p)))
        .map_err(error_to_response)
}
