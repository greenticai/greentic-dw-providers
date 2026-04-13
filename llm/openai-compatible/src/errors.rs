use greentic_dw_llm::{LlmError, LlmErrorKind};
use thiserror::Error;

/// Compatibility-provider-specific error wrapper.
#[derive(Debug, Error)]
pub enum OpenAiCompatibleError {
    /// Invalid provider configuration.
    #[error("{0}")]
    Config(String),
}

impl OpenAiCompatibleError {
    /// Creates a configuration error.
    #[must_use]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }
}

impl From<OpenAiCompatibleError> for LlmError {
    fn from(value: OpenAiCompatibleError) -> Self {
        match value {
            OpenAiCompatibleError::Config(message) => {
                LlmError::new(LlmErrorKind::InvalidRequest, message)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OpenAiCompatibleError;
    use greentic_dw_llm::LlmErrorKind;

    #[test]
    fn config_error_converts_to_invalid_request() {
        let err: greentic_dw_llm::LlmError = OpenAiCompatibleError::config("bad config").into();
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
        assert_eq!(err.message, "bad config");
    }
}
