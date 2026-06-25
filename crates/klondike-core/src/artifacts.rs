use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub source_type: String,
    pub source_location: String,
    pub content_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateArtifact {
    pub name: String,
    pub version: String,
    pub source_type: String,
    pub source_location: String,
    pub content_type: String,
    #[serde(default)]
    pub content: Option<Vec<u8>>,
}
