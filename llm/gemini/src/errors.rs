use greentic_dw_llm::{LlmError, LlmErrorKind};
use thiserror::Error;

/// Gemini-provider-specific error wrapper.
#[derive(Debug, Error)]
pub enum GeminiError {
    /// Invalid provider configuration.
    #[error("{0}")]
    Config(String),
    /// Gemini `generateContent` request failed.
    #[error("Gemini generateContent request failed: {0}")]
    Api(String),
}

impl GeminiError {
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

impl From<GeminiError> for LlmError {
    fn from(value: GeminiError) -> Self {
        let message = value.to_string();
        match value {
            GeminiError::Config(_) => LlmError::new(LlmErrorKind::InvalidRequest, message),
            GeminiError::Api(_) => LlmError::new(LlmErrorKind::Provider, message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GeminiError;
    use greentic_dw_llm::LlmErrorKind;

    #[test]
    fn config_error_converts_to_invalid_request() {
        let err: greentic_dw_llm::LlmError = GeminiError::config("bad config").into();
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
        assert_eq!(err.message, "bad config");
    }

    #[test]
    fn api_error_converts_to_provider_error_with_prefix() {
        let err: greentic_dw_llm::LlmError = GeminiError::api("upstream failed").into();
        assert_eq!(err.kind, LlmErrorKind::Provider);
        assert!(
            err.message
                .contains("Gemini generateContent request failed")
        );
    }
}
