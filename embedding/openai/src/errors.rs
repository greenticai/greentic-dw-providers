use greentic_dw_embedding::{EmbeddingError, EmbeddingErrorKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpenAiEmbeddingError {
    #[error("{0}")]
    Config(String),
}

impl OpenAiEmbeddingError {
    #[must_use]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }
}

impl From<OpenAiEmbeddingError> for EmbeddingError {
    fn from(value: OpenAiEmbeddingError) -> Self {
        match value {
            OpenAiEmbeddingError::Config(message) => {
                EmbeddingError::new(EmbeddingErrorKind::InvalidRequest, message)
            }
        }
    }
}
