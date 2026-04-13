use greentic_dw_llm::{LlmError, LlmErrorKind};
use thiserror::Error;

/// OpenAI-provider-specific error wrapper.
#[derive(Debug, Error)]
pub enum OpenAiError {
    /// Invalid provider configuration.
    #[error("{0}")]
    Config(String),
}

impl OpenAiError {
    /// Creates a configuration error.
    #[must_use]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }
}

impl From<OpenAiError> for LlmError {
    fn from(value: OpenAiError) -> Self {
        match value {
            OpenAiError::Config(message) => LlmError::new(LlmErrorKind::InvalidRequest, message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OpenAiError;
    use greentic_dw_llm::LlmErrorKind;

    #[test]
    fn config_error_converts_to_invalid_request() {
        let err: greentic_dw_llm::LlmError = OpenAiError::config("bad config").into();
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
        assert_eq!(err.message, "bad config");
    }
}
