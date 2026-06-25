use thiserror::Error as ThisError;
use uuid::Uuid;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("not found: {resource} {id}")]
    NotFound { resource: &'static str, id: Uuid },

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}
