use greentic_dw_llm::{LlmError, LlmErrorKind};
use thiserror::Error;

/// Bedrock-provider-specific error wrapper.
#[derive(Debug, Error)]
pub enum BedrockError {
    /// Invalid provider configuration.
    #[error("{0}")]
    Config(String),
    /// Bedrock Converse request failed.
    #[error("Bedrock Converse request failed: {0}")]
    Api(String),
}

impl BedrockError {
    /// Creates a configuration error.
    #[must_use]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Creates an API error.
    #[must_use]
    pub fn api(message: impl Into<String>) -> Self {
        Self::Api(message.into())
    }
}

impl From<BedrockError> for LlmError {
    fn from(value: BedrockError) -> Self {
        let message = value.to_string();
        match value {
            BedrockError::Config(_) => LlmError::new(LlmErrorKind::InvalidRequest, message),
            BedrockError::Api(_) => LlmError::new(LlmErrorKind::Provider, message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BedrockError;
    use greentic_dw_llm::LlmErrorKind;

    #[test]
    fn config_error_converts_to_invalid_request() {
        let err: greentic_dw_llm::LlmError = BedrockError::config("bad config").into();
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
        assert_eq!(err.message, "bad config");
    }

    #[test]
    fn api_error_converts_to_provider_error_with_prefix() {
        let err: greentic_dw_llm::LlmError = BedrockError::api("upstream failed").into();
        assert_eq!(err.kind, LlmErrorKind::Provider);
        assert!(err.message.contains("Bedrock Converse request failed"));
    }
}
