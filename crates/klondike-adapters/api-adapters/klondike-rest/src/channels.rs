use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use klondike_core::api::ChannelsApi;
use klondike_core::channels::{Channel, CreateChannel};

use super::error_to_response;

pub fn routes<S: ChannelsApi + 'static>(state: Arc<S>) -> Router {
    Router::new()
        .route("/api/v1/channels", get(list).post(create))
        .route("/api/v1/channels/:id", get(get_one))
        .with_state(state)
}

async fn list<S: ChannelsApi>(
    State(state): State<Arc<S>>,
) -> Result<Json<Vec<Channel>>, axum::http::StatusCode> {
    state.list_channels().await.map(Json).map_err(error_to_response)
}

async fn get_one<S: ChannelsApi>(
    State(state): State<Arc<S>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Channel>, axum::http::StatusCode> {
    state.get_channel(id).await.map(Json).map_err(error_to_response)
}

async fn create<S: ChannelsApi>(
    State(state): State<Arc<S>>,
    Json(input): Json<CreateChannel>,
) -> Result<(axum::http::StatusCode, Json<Channel>), axum::http::StatusCode> {
    state
        .create_channel(input)
        .await
        .map(|c| (axum::http::StatusCode::CREATED, Json(c)))
        .map_err(error_to_response)
}
