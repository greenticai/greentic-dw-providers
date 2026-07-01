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

#[cfg(test)]
mod tests {
    use super::OpenAiCompatibleEmbeddingError;
    use greentic_dw_embedding::{EmbeddingError, EmbeddingErrorKind};

    #[test]
    fn config_constructor_carries_message() {
        let err = OpenAiCompatibleEmbeddingError::config("bad config");
        assert_eq!(err.to_string(), "bad config");
    }

    #[test]
    fn maps_into_embedding_invalid_request() {
        let mapped: EmbeddingError = OpenAiCompatibleEmbeddingError::config("bad config").into();
        assert_eq!(mapped.kind, EmbeddingErrorKind::InvalidRequest);
        assert_eq!(mapped.message, "bad config");
    }
}
