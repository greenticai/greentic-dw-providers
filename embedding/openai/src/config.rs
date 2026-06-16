use reqwest::Url;
use serde::{Deserialize, Serialize};

use greentic_dw_embedding::{DEFAULT_EMBEDDING_DIM, DEFAULT_EMBEDDING_MODEL};

use crate::errors::OpenAiEmbeddingError;

pub const OPENAI_DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";

/// Native OpenAI embedding configuration. `api_key_secret` is a secret
/// reference resolved by the runtime host, never a bare key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenAiEmbeddingConfig {
    pub api_key_secret: String,
    pub base_url: Option<String>,
    pub model: String,
    pub embedding_dim: usize,
    pub timeout_ms: u64,
}

impl OpenAiEmbeddingConfig {
    #[must_use]
    pub fn new(
        api_key_secret: impl Into<String>,
        model: impl Into<String>,
        timeout_ms: u64,
    ) -> Self {
        Self {
            api_key_secret: api_key_secret.into(),
            base_url: None,
            model: model.into(),
            embedding_dim: DEFAULT_EMBEDDING_DIM,
            timeout_ms,
        }
    }

    #[must_use]
    pub fn with_defaults(api_key_secret: impl Into<String>, timeout_ms: u64) -> Self {
        Self::new(api_key_secret, DEFAULT_EMBEDDING_MODEL, timeout_ms)
    }

    pub fn validate(&self) -> Result<(), OpenAiEmbeddingError> {
        if self.api_key_secret.trim().is_empty() {
            return Err(OpenAiEmbeddingError::config(
                "api_key_secret must not be empty",
            ));
        }
        if self.model.trim().is_empty() {
            return Err(OpenAiEmbeddingError::config("model must not be empty"));
        }
        if self.embedding_dim == 0 {
            return Err(OpenAiEmbeddingError::config(
                "embedding_dim must be greater than zero",
            ));
        }
        if self.timeout_ms == 0 {
            return Err(OpenAiEmbeddingError::config(
                "timeout_ms must be greater than zero",
            ));
        }
        if let Some(base_url) = &self.base_url {
            validate_https_override(base_url)?;
        }
        Ok(())
    }

    #[must_use]
    pub fn base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or(OPENAI_DEFAULT_BASE_URL)
            .trim_end_matches('/')
    }

    #[must_use]
    pub fn embeddings_url(&self) -> String {
        format!("{}/embeddings", self.base_url())
    }
}

fn validate_https_override(base_url: &str) -> Result<(), OpenAiEmbeddingError> {
    let parsed = Url::parse(base_url)
        .map_err(|_| OpenAiEmbeddingError::config("base_url must be a valid absolute URL"))?;
    if parsed.scheme() != "https" {
        return Err(OpenAiEmbeddingError::config(
            "base_url must use https for the native OpenAI provider",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_empty_key() {
        let cfg = OpenAiEmbeddingConfig::new("  ", "text-embedding-3-small", 10_000);
        assert_eq!(
            cfg.validate().expect_err("empty key").to_string(),
            "api_key_secret must not be empty"
        );
    }

    #[test]
    fn validate_rejects_zero_dim() {
        let mut cfg = OpenAiEmbeddingConfig::with_defaults("sk-test", 10_000);
        cfg.embedding_dim = 0;
        assert_eq!(
            cfg.validate().expect_err("zero dim").to_string(),
            "embedding_dim must be greater than zero"
        );
    }

    #[test]
    fn validate_rejects_non_https_base_url() {
        let mut cfg = OpenAiEmbeddingConfig::with_defaults("sk-test", 10_000);
        cfg.base_url = Some("http://example.com/v1".to_string());
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn embeddings_url_trims_and_appends() {
        let mut cfg = OpenAiEmbeddingConfig::with_defaults("sk-test", 10_000);
        cfg.base_url = Some("https://proxy.example.com/v1/".to_string());
        assert_eq!(
            cfg.embeddings_url(),
            "https://proxy.example.com/v1/embeddings"
        );
    }
}
