use std::time::Duration;

use greentic_dw_embedding::{EmbeddingError, EmbeddingErrorKind, EmbeddingResult};
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::Value;

use crate::config::OpenAiCompatibleEmbeddingConfig;

/// Transport seam (mockable in tests).
pub trait OpenAiCompatibleEmbeddingTransport: Send + Sync {
    fn create_embeddings(
        &self,
        config: &OpenAiCompatibleEmbeddingConfig,
        payload: &Value,
    ) -> EmbeddingResult<Value>;
}

#[derive(Default)]
pub struct HttpOpenAiCompatibleEmbeddingTransport;

impl OpenAiCompatibleEmbeddingTransport for HttpOpenAiCompatibleEmbeddingTransport {
    fn create_embeddings(
        &self,
        config: &OpenAiCompatibleEmbeddingConfig,
        payload: &Value,
    ) -> EmbeddingResult<Value> {
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .map_err(|err| EmbeddingError::new(EmbeddingErrorKind::Internal, err.to_string()))?;

        let mut headers = HeaderMap::new();
        let auth =
            HeaderValue::from_str(&format!("Bearer {}", config.api_key_secret)).map_err(|err| {
                EmbeddingError::new(EmbeddingErrorKind::InvalidRequest, err.to_string())
            })?;
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
    if err.is_connect() {
        return EmbeddingError::new(EmbeddingErrorKind::Network, err.to_string()).retryable(true);
    }
    if err.is_request() {
        return EmbeddingError::new(EmbeddingErrorKind::Network, err.to_string());
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
    EmbeddingError::new(
        kind,
        format!("openai-compatible embeddings error {status}: {body}"),
    )
    .retryable(retryable)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        HttpOpenAiCompatibleEmbeddingTransport, OpenAiCompatibleEmbeddingTransport, map_http_error,
    };
    use crate::config::OpenAiCompatibleEmbeddingConfig;
    use greentic_dw_embedding::EmbeddingErrorKind;
    use serde_json::json;

    #[test]
    fn map_http_error_classifies_status_codes() {
        assert_eq!(map_http_error(401, "nope").kind, EmbeddingErrorKind::Auth);
        assert_eq!(map_http_error(403, "nope").kind, EmbeddingErrorKind::Auth);

        let rate_limited = map_http_error(429, "slow down");
        assert_eq!(rate_limited.kind, EmbeddingErrorKind::RateLimited);
        assert!(rate_limited.retryable);

        let server = map_http_error(500, "boom");
        assert_eq!(server.kind, EmbeddingErrorKind::Provider);
        assert!(server.retryable);

        let other = map_http_error(400, "bad body");
        assert_eq!(other.kind, EmbeddingErrorKind::Provider);
        assert!(!other.retryable);
        assert!(
            other
                .message
                .contains("openai-compatible embeddings error 400")
        );
    }

    #[test]
    fn create_embeddings_fails_before_network_for_invalid_api_key() {
        let config = OpenAiCompatibleEmbeddingConfig::new(
            "bad\nkey",
            "http://localhost:11434/v1",
            "text-embedding-3-small",
            5_000,
        );
        let err = HttpOpenAiCompatibleEmbeddingTransport
            .create_embeddings(&config, &json!({ "input": "hello" }))
            .expect_err("invalid header must fail before any network call");
        assert_eq!(err.kind, EmbeddingErrorKind::InvalidRequest);
    }
}
