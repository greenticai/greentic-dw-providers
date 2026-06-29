use greentic_dw_embedding::{EmbeddingError, EmbeddingErrorKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenAiCompatibleEmbeddingError {
    #[error("{0}")]
    Config(String),
}

impl OpenAiCompatibleEmbeddingError {
    #[must_use]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }
}

impl From<OpenAiCompatibleEmbeddingError> for EmbeddingError {
    fn from(value: OpenAiCompatibleEmbeddingError) -> Self {
        match value {
            OpenAiCompatibleEmbeddingError::Config(message) => {
                EmbeddingError::new(EmbeddingErrorKind::InvalidRequest, message)
            }
        }
    }
}
