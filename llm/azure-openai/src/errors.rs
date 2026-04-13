use greentic_dw_llm::{LlmError, LlmErrorKind};
use thiserror::Error;

/// Azure OpenAI-provider-specific error wrapper.
#[derive(Debug, Error)]
pub enum AzureOpenAiError {
    /// Invalid provider configuration.
    #[error("{0}")]
    Config(String),
}

impl AzureOpenAiError {
    /// Creates a configuration error.
    #[must_use]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }
}

impl From<AzureOpenAiError> for LlmError {
    fn from(value: AzureOpenAiError) -> Self {
        match value {
            AzureOpenAiError::Config(message) => {
                LlmError::new(LlmErrorKind::InvalidRequest, message)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AzureOpenAiError;
    use greentic_dw_llm::LlmErrorKind;

    #[test]
    fn config_error_converts_to_invalid_request() {
        let err: greentic_dw_llm::LlmError = AzureOpenAiError::config("bad config").into();
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
        assert_eq!(err.message, "bad config");
    }
}
