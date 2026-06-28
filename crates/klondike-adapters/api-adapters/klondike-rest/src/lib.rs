mod channels;
mod threads;
mod posts;
mod issues;
mod artifacts;
mod users;

use std::sync::Arc;

use axum::Router;
use klondike_core::api::*;
use klondike_core::Error;

pub fn router<S>(state: Arc<S>) -> Router
where
    S: ChannelsApi + ThreadsApi + PostsApi + IssuesApi + ArtifactsApi + UsersApi + 'static,
{
    Router::new()
        .merge(channels::routes(Arc::clone(&state)))
        .merge(threads::routes(Arc::clone(&state)))
        .merge(posts::routes(Arc::clone(&state)))
        .merge(issues::routes(Arc::clone(&state)))
        .merge(artifacts::routes(Arc::clone(&state)))
        .merge(users::routes(state))
}

fn error_to_response(err: Error) -> axum::http::StatusCode {
    match err {
        Error::NotFound { .. } => axum::http::StatusCode::NOT_FOUND,
        Error::InvalidInput(_) => axum::http::StatusCode::BAD_REQUEST,
        _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    }
}
