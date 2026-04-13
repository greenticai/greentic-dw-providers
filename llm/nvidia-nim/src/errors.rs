use greentic_dw_llm::{LlmError, LlmErrorKind};
use thiserror::Error;

/// NVIDIA NIM-provider-specific error wrapper.
#[derive(Debug, Error)]
pub enum NvidiaNimError {
    /// Invalid provider configuration.
    #[error("{0}")]
    Config(String),
    /// Inference request failed against the OpenAI-compatible NIM surface.
    #[error("NIM inference request failed: {0}")]
    Inference(String),
    /// Discovery request failed against the NIM management surface.
    #[error("NIM discovery request failed: {0}")]
    Discovery(String),
    /// Health request failed against the NIM management surface.
    #[error("NIM health request failed: {0}")]
    Health(String),
}

impl NvidiaNimError {
    /// Creates a configuration error.
    #[must_use]
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Creates an inference error.
    #[must_use]
    pub fn inference(message: impl Into<String>) -> Self {
        Self::Inference(message.into())
    }

    /// Creates a discovery error.
    #[must_use]
    pub fn discovery(message: impl Into<String>) -> Self {
        Self::Discovery(message.into())
    }

    /// Creates a health error.
    #[must_use]
    pub fn health(message: impl Into<String>) -> Self {
        Self::Health(message.into())
    }
}

impl From<NvidiaNimError> for LlmError {
    fn from(value: NvidiaNimError) -> Self {
        let message = value.to_string();
        match value {
            NvidiaNimError::Config(_) => LlmError::new(LlmErrorKind::InvalidRequest, message),
            NvidiaNimError::Inference(_)
            | NvidiaNimError::Discovery(_)
            | NvidiaNimError::Health(_) => LlmError::new(LlmErrorKind::Provider, message),
        }
    }
}
