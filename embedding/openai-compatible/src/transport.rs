use std::time::Duration;

use greentic_dw_embedding::{EmbeddingError, EmbeddingErrorKind, EmbeddingResult};
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::Value;

use crate::config::OpenAiCompatibleEmbeddingConfig;

/// Transport seam (mockable in tests).
pub trait OpenAiCompatibleEmbeddingTransport: Send + Sync {
    fn create_embeddings(&self, config: &OpenAiCompatibleEmbeddingConfig, payload: &Value) -> EmbeddingResult<Value>;
}

#[derive(Default)]
pub struct HttpOpenAiCompatibleEmbeddingTransport;

impl OpenAiCompatibleEmbeddingTransport for HttpOpenAiCompatibleEmbeddingTransport {
    fn create_embeddings(&self, config: &OpenAiCompatibleEmbeddingConfig, payload: &Value) -> EmbeddingResult<Value> {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|err| EmbeddingError::new(EmbeddingErrorKind::Internal, err.to_string()))?;

        let mut headers = HeaderMap::new();
        let auth = HeaderValue::from_str(&format!("Bearer {}", config.api_key_secret))
            .map_err(|err| EmbeddingError::new(EmbeddingErrorKind::InvalidRequest, err.to_string()))?;
        headers.insert(AUTHORIZATION, auth);
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let response = client
            .post(config.embeddings_url())
            .headers(headers)
            .json(payload)
            .send()
            .map_err(map_reqwest_error)?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|err| EmbeddingError::new(EmbeddingErrorKind::Network, err.to_string()))?;

        if !status.is_success() {
            return Err(map_http_error(status.as_u16(), &body));
        }
        serde_json::from_str(&body)
            .map_err(|err| EmbeddingError::new(EmbeddingErrorKind::Provider, err.to_string()))
    }
}

fn map_reqwest_error(err: reqwest::Error) -> EmbeddingError {
    if err.is_timeout() {
        return EmbeddingError::new(EmbeddingErrorKind::Timeout, err.to_string()).retryable(true);
    }
    if err.is_connect() || err.is_request() {
        return EmbeddingError::new(EmbeddingErrorKind::Network, err.to_string()).retryable(true);
    }
    EmbeddingError::new(EmbeddingErrorKind::Provider, err.to_string())
}

/// Maps a non-2xx HTTP status to a normalized error.
pub(crate) fn map_http_error(status: u16, body: &str) -> EmbeddingError {
    let kind = match status {
        401 | 403 => EmbeddingErrorKind::Auth,
        429 => EmbeddingErrorKind::RateLimited,
        500..=599 => EmbeddingErrorKind::Provider,
        _ => EmbeddingErrorKind::Provider,
    };
    let retryable = matches!(status, 429 | 500..=599);
    EmbeddingError::new(kind, format!("openai-compatible embeddings error {status}: {body}")).retryable(retryable)
}
