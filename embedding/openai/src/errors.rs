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

#[cfg(test)]
mod tests {
    use super::OpenAiEmbeddingError;
    use greentic_dw_embedding::{EmbeddingError, EmbeddingErrorKind};

    #[test]
    fn config_constructor_carries_message() {
        let err = OpenAiEmbeddingError::config("bad config");
        assert_eq!(err.to_string(), "bad config");
    }

    #[test]
    fn maps_into_embedding_invalid_request() {
        let mapped: EmbeddingError = OpenAiEmbeddingError::config("bad config").into();
        assert_eq!(mapped.kind, EmbeddingErrorKind::InvalidRequest);
        assert_eq!(mapped.message, "bad config");
    }
}
