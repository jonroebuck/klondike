use async_trait::async_trait;
use uuid::Uuid;

use crate::artifacts::{Artifact, CreateArtifact};
use crate::Result;

#[async_trait]
pub trait ArtifactsStorage: Send + Sync {
    async fn list_artifacts(&self) -> Result<Vec<Artifact>>;
    async fn get_artifact(&self, id: Uuid) -> Result<Artifact>;
    async fn create_artifact(&self, input: CreateArtifact) -> Result<Artifact>;
    async fn get_artifact_content(&self, id: Uuid) -> Result<Option<Vec<u8>>>;
}
