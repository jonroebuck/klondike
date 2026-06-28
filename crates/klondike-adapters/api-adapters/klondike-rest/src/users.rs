use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use klondike_core::api::UsersApi;
use klondike_core::users::{ChannelSubscription, FeedPost, ThreadSubscription, User};

use super::error_to_response;

#[derive(Deserialize)]
struct SubscribeChannelBody {
    channel_id: Uuid,
}

#[derive(Deserialize)]
struct SubscribeThreadBody {
    channel_id: Uuid,
    thread_id: Uuid,
}

#[derive(Serialize)]
struct SubscriptionsResponse {
    channels: Vec<ChannelSubscription>,
    threads: Vec<ThreadSubscription>,
}

pub fn routes<S: UsersApi + 'static>(state: Arc<S>) -> Router {
    Router::new()
        .route("/api/v1/users/register", post(register))
        .route("/api/v1/users/:user_id", delete(unregister))
        .route(
            "/api/v1/users/:user_id/subscriptions/channels",
            post(subscribe_channel),
        )
        .route(
            "/api/v1/users/:user_id/subscriptions/channels/:channel_id",
            delete(unsubscribe_channel),
        )
        .route(
            "/api/v1/users/:user_id/subscriptions/threads",
            post(subscribe_thread),
        )
        .route(
            "/api/v1/users/:user_id/subscriptions/threads/:thread_id",
            delete(unsubscribe_thread),
        )
        .route("/api/v1/users/:user_id/feed", get(feed))
        .route("/api/v1/users/:user_id/subscriptions", get(list_subscriptions))
        .with_state(state)
}

async fn register<S: UsersApi>(
    State(state): State<Arc<S>>,
) -> Result<(axum::http::StatusCode, Json<User>), axum::http::StatusCode> {
    state
        .register_user()
        .await
        .map(|u| (axum::http::StatusCode::CREATED, Json(u)))
        .map_err(error_to_response)
}

async fn unregister<S: UsersApi>(
    State(state): State<Arc<S>>,
    Path(user_id): Path<Uuid>,
) -> Result<axum::http::StatusCode, axum::http::StatusCode> {
    state
        .unregister_user(user_id)
        .await
        .map(|_| axum::http::StatusCode::NO_CONTENT)
        .map_err(error_to_response)
}

async fn subscribe_channel<S: UsersApi>(
    State(state): State<Arc<S>>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<SubscribeChannelBody>,
) -> Result<(axum::http::StatusCode, Json<ChannelSubscription>), axum::http::StatusCode> {
    state
        .subscribe_to_channel(user_id, body.channel_id)
        .await
        .map(|s| (axum::http::StatusCode::CREATED, Json(s)))
        .map_err(error_to_response)
}

async fn unsubscribe_channel<S: UsersApi>(
    State(state): State<Arc<S>>,
    Path((user_id, channel_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, axum::http::StatusCode> {
    state
        .unsubscribe_from_channel(user_id, channel_id)
        .await
        .map(|_| axum::http::StatusCode::NO_CONTENT)
        .map_err(error_to_response)
}

async fn subscribe_thread<S: UsersApi>(
    State(state): State<Arc<S>>,
    Path(user_id): Path<Uuid>,
    Json(body): Json<SubscribeThreadBody>,
) -> Result<(axum::http::StatusCode, Json<ThreadSubscription>), axum::http::StatusCode> {
    state
        .subscribe_to_thread(user_id, body.channel_id, body.thread_id)
        .await
        .map(|s| (axum::http::StatusCode::CREATED, Json(s)))
        .map_err(error_to_response)
}

async fn unsubscribe_thread<S: UsersApi>(
    State(state): State<Arc<S>>,
    Path((user_id, thread_id)): Path<(Uuid, Uuid)>,
) -> Result<axum::http::StatusCode, axum::http::StatusCode> {
    state
        .unsubscribe_from_thread(user_id, thread_id)
        .await
        .map(|_| axum::http::StatusCode::NO_CONTENT)
        .map_err(error_to_response)
}

async fn feed<S: UsersApi>(
    State(state): State<Arc<S>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<FeedPost>>, axum::http::StatusCode> {
    state
        .get_feed(user_id)
        .await
        .map(Json)
        .map_err(error_to_response)
}

async fn list_subscriptions<S: UsersApi>(
    State(state): State<Arc<S>>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<SubscriptionsResponse>, axum::http::StatusCode> {
    state
        .list_subscriptions(user_id)
        .await
        .map(|(channels, threads)| Json(SubscriptionsResponse { channels, threads }))
        .map_err(error_to_response)
}
