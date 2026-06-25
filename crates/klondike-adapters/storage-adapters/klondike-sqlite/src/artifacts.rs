use async_trait::async_trait;
use chrono::Utc;
use sqlx::Row;
use uuid::Uuid;

use klondike_core::artifacts::{Artifact, CreateArtifact};
use klondike_core::storage::ArtifactsStorage;
use klondike_core::{Error, Result};

use crate::SqliteStorage;

fn row_to_artifact(row: sqlx::sqlite::SqliteRow) -> std::result::Result<Artifact, sqlx::Error> {
    Ok(Artifact {
        id: row.try_get::<String, _>("id")?.parse().map_err(|e: uuid::Error| sqlx::Error::Decode(e.into()))?,
        name: row.try_get("name")?,
        version: row.try_get("version")?,
        source_type: row.try_get("source_type")?,
        source_location: row.try_get("source_location")?,
        content_type: row.try_get("content_type")?,
        created_at: row.try_get::<String, _>("created_at")?.parse().map_err(|e: chrono::ParseError| sqlx::Error::Decode(e.into()))?,
    })
}

#[async_trait]
impl ArtifactsStorage for SqliteStorage {
    async fn list_artifacts(&self) -> Result<Vec<Artifact>> {
        let rows = sqlx::query("SELECT id, name, version, source_type, source_location, content_type, created_at FROM artifacts ORDER BY created_at")
            .fetch_all(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        rows.into_iter()
            .map(|r| row_to_artifact(r).map_err(|e| Error::Storage(e.to_string())))
            .collect()
    }

    async fn get_artifact(&self, id: Uuid) -> Result<Artifact> {
        let row = sqlx::query("SELECT id, name, version, source_type, source_location, content_type, created_at FROM artifacts WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?
            .ok_or(Error::NotFound { resource: "artifact", id })?;

        row_to_artifact(row).map_err(|e| Error::Storage(e.to_string()))
    }

    async fn create_artifact(&self, input: CreateArtifact) -> Result<Artifact> {
        let artifact = Artifact {
            id: Uuid::new_v4(),
            name: input.name,
            version: input.version,
            source_type: input.source_type,
            source_location: input.source_location,
            content_type: input.content_type,
            created_at: Utc::now(),
        };

        sqlx::query("INSERT INTO artifacts (id, name, version, source_type, source_location, content_type, content, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)")
            .bind(artifact.id.to_string())
            .bind(&artifact.name)
            .bind(&artifact.version)
            .bind(&artifact.source_type)
            .bind(&artifact.source_location)
            .bind(&artifact.content_type)
            .bind(input.content.as_deref())
            .bind(artifact.created_at.to_rfc3339())
            .execute(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?;

        Ok(artifact)
    }

    async fn get_artifact_content(&self, id: Uuid) -> Result<Option<Vec<u8>>> {
        let row = sqlx::query("SELECT content FROM artifacts WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(self.pool())
            .await
            .map_err(|e| Error::Storage(e.to_string()))?
            .ok_or(Error::NotFound { resource: "artifact", id })?;

        Ok(row.try_get::<Option<Vec<u8>>, _>("content")
            .map_err(|e| Error::Storage(e.to_string()))?)
    }
}
