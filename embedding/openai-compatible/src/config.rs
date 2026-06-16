use reqwest::Url;
use serde::{Deserialize, Serialize};

use greentic_dw_embedding::{DEFAULT_EMBEDDING_DIM, DEFAULT_EMBEDDING_MODEL};

use crate::errors::OpenAiCompatibleEmbeddingError;

/// OpenAI-compatible embedding configuration. `base_url` is required (no
/// default) and accepts both `http` and `https` schemes for local servers such
/// as Ollama or vLLM. `api_key_secret` is a secret reference resolved by the
/// runtime host, never a bare key.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenAiCompatibleEmbeddingConfig {
    pub api_key_secret: String,
    pub base_url: String,
    pub model: String,
    pub embedding_dim: usize,
    pub timeout_ms: u64,
}

impl OpenAiCompatibleEmbeddingConfig {
    #[must_use]
    pub fn new(
        api_key_secret: impl Into<String>,
        base_url: impl Into<String>,
        model: impl Into<String>,
        timeout_ms: u64,
    ) -> Self {
        Self {
            api_key_secret: api_key_secret.into(),
            base_url: base_url.into(),
            model: model.into(),
            embedding_dim: DEFAULT_EMBEDDING_DIM,
            timeout_ms,
        }
    }

    #[must_use]
    pub fn with_defaults(
        api_key_secret: impl Into<String>,
        base_url: impl Into<String>,
        timeout_ms: u64,
    ) -> Self {
        Self::new(api_key_secret, base_url, DEFAULT_EMBEDDING_MODEL, timeout_ms)
    }

    pub fn validate(&self) -> Result<(), OpenAiCompatibleEmbeddingError> {
        if self.api_key_secret.trim().is_empty() {
            return Err(OpenAiCompatibleEmbeddingError::config("api_key_secret must not be empty"));
        }
        if self.base_url.trim().is_empty() {
            return Err(OpenAiCompatibleEmbeddingError::config("base_url must not be empty"));
        }
        validate_base_url(&self.base_url)?;
        if self.model.trim().is_empty() {
            return Err(OpenAiCompatibleEmbeddingError::config("model must not be empty"));
        }
        if self.embedding_dim == 0 {
            return Err(OpenAiCompatibleEmbeddingError::config("embedding_dim must be greater than zero"));
        }
        if self.timeout_ms == 0 {
            return Err(OpenAiCompatibleEmbeddingError::config("timeout_ms must be greater than zero"));
        }
        Ok(())
    }

    #[must_use]
    pub fn embeddings_url(&self) -> String {
        format!("{}/embeddings", self.base_url.trim_end_matches('/'))
    }
}

fn validate_base_url(base_url: &str) -> Result<(), OpenAiCompatibleEmbeddingError> {
    let parsed = Url::parse(base_url)
        .map_err(|_| OpenAiCompatibleEmbeddingError::config("base_url must be a valid absolute URL"))?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(OpenAiCompatibleEmbeddingError::config(
            "base_url must use http or https",
        ));
    }
    Ok(())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn validate_requires_base_url() {
        let cfg = OpenAiCompatibleEmbeddingConfig::new("sk-test", "  ", "text-embedding-3-small", 10_000);
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn allows_http_for_local_endpoints() {
        let cfg = OpenAiCompatibleEmbeddingConfig::with_defaults(
            "sk-test",
            "http://localhost:11434/v1",
            10_000,
        );
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn validate_rejects_empty_key() {
        let cfg = OpenAiCompatibleEmbeddingConfig::new("  ", "http://localhost:11434/v1", "text-embedding-3-small", 10_000);
        assert_eq!(cfg.validate().expect_err("empty key").to_string(), "api_key_secret must not be empty");
    }

    #[test]
    fn validate_rejects_zero_dim() {
        let mut cfg = OpenAiCompatibleEmbeddingConfig::with_defaults("sk-test", "http://localhost:11434/v1", 10_000);
        cfg.embedding_dim = 0;
        assert_eq!(cfg.validate().expect_err("zero dim").to_string(), "embedding_dim must be greater than zero");
    }

    #[test]
    fn validate_rejects_invalid_scheme() {
        let cfg = OpenAiCompatibleEmbeddingConfig::new("sk-test", "ftp://example.com/v1", "text-embedding-3-small", 10_000);
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn embeddings_url_trims_and_appends() {
        let cfg = OpenAiCompatibleEmbeddingConfig::with_defaults("sk-test", "https://proxy.example.com/v1/", 10_000);
        assert_eq!(cfg.embeddings_url(), "https://proxy.example.com/v1/embeddings");
    }
}
