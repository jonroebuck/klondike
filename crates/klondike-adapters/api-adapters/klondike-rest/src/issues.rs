use std::sync::Arc;

use axum::extract::{Path, State};
use axum::routing::{get, patch};
use axum::{Json, Router};
use uuid::Uuid;

use klondike_core::api::IssuesApi;
use klondike_core::issues::{CreateIssue, Issue, IssueEvent, UpdateIssueStatus};

use super::error_to_response;

pub fn routes<S: IssuesApi + 'static>(state: Arc<S>) -> Router {
    Router::new()
        .route("/api/v1/issues", get(list).post(create))
        .route("/api/v1/issues/{id}", get(get_one))
        .route("/api/v1/issues/{id}/status", patch(update_status))
        .route("/api/v1/issues/{id}/events", get(list_events))
        .with_state(state)
}

async fn list<S: IssuesApi>(
    State(state): State<Arc<S>>,
) -> Result<Json<Vec<Issue>>, axum::http::StatusCode> {
    state.list_issues().await.map(Json).map_err(error_to_response)
}

async fn get_one<S: IssuesApi>(
    State(state): State<Arc<S>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Issue>, axum::http::StatusCode> {
    state.get_issue(id).await.map(Json).map_err(error_to_response)
}

async fn create<S: IssuesApi>(
    State(state): State<Arc<S>>,
    Json(input): Json<CreateIssue>,
) -> Result<(axum::http::StatusCode, Json<Issue>), axum::http::StatusCode> {
    state
        .create_issue(input)
        .await
        .map(|i| (axum::http::StatusCode::CREATED, Json(i)))
        .map_err(error_to_response)
}

async fn update_status<S: IssuesApi>(
    State(state): State<Arc<S>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateIssueStatus>,
) -> Result<Json<Issue>, axum::http::StatusCode> {
    state.update_status(id, input).await.map(Json).map_err(error_to_response)
}

async fn list_events<S: IssuesApi>(
    State(state): State<Arc<S>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<IssueEvent>>, axum::http::StatusCode> {
    state.list_events(id).await.map(Json).map_err(error_to_response)
}
