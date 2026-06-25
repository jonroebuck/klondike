use async_trait::async_trait;
use uuid::Uuid;

use crate::artifacts::{Artifact, CreateArtifact};
use crate::Result;

#[async_trait]
pub trait ArtifactsApi: Send + Sync {
    async fn list_artifacts(&self) -> Result<Vec<Artifact>>;
    async fn get_artifact(&self, id: Uuid) -> Result<Artifact>;
    async fn create_artifact(&self, input: CreateArtifact) -> Result<Artifact>;
}
