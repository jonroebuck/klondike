use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use klondike_core::api::ArtifactsApi;
use klondike_core::artifacts::{Artifact, CreateArtifact};

use super::error_to_response;

pub fn routes<S: ArtifactsApi + 'static>(state: Arc<S>) -> Router {
    Router::new()
        .route("/api/v1/artifacts", get(list).post(create))
        .route("/api/v1/artifacts/:id", get(get_one))
        .with_state(state)
}

async fn list<S: ArtifactsApi>(
    State(state): State<Arc<S>>,
) -> Result<Json<Vec<Artifact>>, axum::http::StatusCode> {
    state.list_artifacts().await.map(Json).map_err(error_to_response)
}

async fn get_one<S: ArtifactsApi>(
    State(state): State<Arc<S>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Artifact>, axum::http::StatusCode> {
    state.get_artifact(id).await.map(Json).map_err(error_to_response)
}

async fn create<S: ArtifactsApi>(
    State(state): State<Arc<S>>,
    Json(input): Json<CreateArtifact>,
) -> Result<(axum::http::StatusCode, Json<Artifact>), axum::http::StatusCode> {
    state
        .create_artifact(input)
        .await
        .map(|a| (axum::http::StatusCode::CREATED, Json(a)))
        .map_err(error_to_response)
}
