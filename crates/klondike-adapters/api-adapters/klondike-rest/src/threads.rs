use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use klondike_core::api::ThreadsApi;
use klondike_core::threads::{CreateThread, Thread};

use super::error_to_response;

pub fn routes<S: ThreadsApi + 'static>(state: Arc<S>) -> Router {
    Router::new()
        .route("/api/v1/channels/:channel_id/threads", get(list).post(create))
        .route("/api/v1/threads/:id", get(get_one))
        .with_state(state)
}

async fn list<S: ThreadsApi>(
    State(state): State<Arc<S>>,
    Path(channel_id): Path<Uuid>,
) -> Result<Json<Vec<Thread>>, axum::http::StatusCode> {
    state.list_threads(channel_id).await.map(Json).map_err(error_to_response)
}

async fn get_one<S: ThreadsApi>(
    State(state): State<Arc<S>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Thread>, axum::http::StatusCode> {
    state.get_thread(id).await.map(Json).map_err(error_to_response)
}

async fn create<S: ThreadsApi>(
    State(state): State<Arc<S>>,
    Path(channel_id): Path<Uuid>,
    Json(input): Json<CreateThread>,
) -> Result<(axum::http::StatusCode, Json<Thread>), axum::http::StatusCode> {
    state
        .create_thread(channel_id, input)
        .await
        .map(|t| (axum::http::StatusCode::CREATED, Json(t)))
        .map_err(error_to_response)
}
