use std::time::Duration;

use greentic_dw_llm::{LlmError, LlmErrorKind, LlmResult};
use reqwest::blocking::Client;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::Value;

use crate::config::AnthropicConfig;
use crate::mapping::map_http_error;

/// Anthropic API version used by this provider.
pub const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// Transport abstraction used to execute Anthropic Messages API calls.
pub trait AnthropicTransport: Send + Sync {
    /// Creates a message for the serialized request payload.
    fn create_message(&self, config: &AnthropicConfig, payload: &Value) -> LlmResult<Value>;
}

/// Default HTTP transport backed by `reqwest`.
#[derive(Default)]
pub struct HttpAnthropicTransport;

impl AnthropicTransport for HttpAnthropicTransport {
    fn create_message(&self, config: &AnthropicConfig, payload: &Value) -> LlmResult<Value> {
        let client = build_client(config.timeout_ms)?;
        let headers = build_headers(config)?;

        let response = client
            .post(config.messages_url())
            .headers(headers)
            .json(payload)
            .send()
            .map_err(map_reqwest_error)?;
        let status = response.status();
        let body = response
            .text()
            .map_err(|err| LlmError::new(LlmErrorKind::Network, err.to_string()))?;

        if !status.is_success() {
            return Err(map_http_error(status.as_u16(), &body));
        }

        serde_json::from_str(&body)
            .map_err(|err| LlmError::new(LlmErrorKind::Provider, err.to_string()))
    }
}

fn build_client(timeout_ms: u64) -> LlmResult<Client> {
    Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .map_err(|err| LlmError::new(LlmErrorKind::Internal, err.to_string()))
}

fn build_headers(config: &AnthropicConfig) -> LlmResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        "x-api-key",
        HeaderValue::from_str(&config.api_key_secret)
            .map_err(|err| LlmError::new(LlmErrorKind::InvalidRequest, err.to_string()))?,
    );
    headers.insert(
        "anthropic-version",
        HeaderValue::from_static(ANTHROPIC_API_VERSION),
    );
    Ok(headers)
}

fn map_reqwest_error(err: reqwest::Error) -> LlmError {
    if err.is_timeout() {
        return LlmError::new(LlmErrorKind::Timeout, err.to_string()).retryable(true);
    }
    if err.is_connect() || err.is_request() {
        return LlmError::new(LlmErrorKind::Network, err.to_string()).retryable(true);
    }
    LlmError::new(LlmErrorKind::Provider, err.to_string())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        ANTHROPIC_API_VERSION, AnthropicTransport, HttpAnthropicTransport, build_client,
        build_headers,
    };
    use crate::config::AnthropicConfig;
    use greentic_dw_llm::LlmErrorKind;
    use reqwest::header::CONTENT_TYPE;
    use serde_json::json;

    fn config() -> AnthropicConfig {
        AnthropicConfig::new("secret", "claude-sonnet", 256, 5_000)
    }

    #[test]
    fn build_headers_includes_api_key_and_version() {
        let headers = build_headers(&config()).expect("headers");
        assert_eq!(headers[CONTENT_TYPE], "application/json");
        assert_eq!(headers["x-api-key"], "secret");
        assert_eq!(headers["anthropic-version"], ANTHROPIC_API_VERSION);
    }

    #[test]
    fn build_headers_rejects_invalid_api_key_values() {
        let mut config = config();
        config.api_key_secret = "bad\nkey".to_string();
        let err = build_headers(&config).expect_err("invalid header");
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
    }

    #[test]
    fn build_client_accepts_timeout() {
        build_client(1).expect("client");
    }

    #[test]
    fn create_message_fails_before_network_for_invalid_api_key_header() {
        let mut config = config();
        config.api_key_secret = "bad\nkey".to_string();
        let err = HttpAnthropicTransport
            .create_message(&config, &json!({"messages":[]}))
            .expect_err("invalid header");
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
    }
}
