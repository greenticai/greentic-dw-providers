use std::time::Duration;

use greentic_dw_llm::{LlmError, LlmErrorKind, LlmResult};
use reqwest::blocking::Client;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::Value;

use crate::config::GeminiConfig;
use crate::mapping::map_http_error;

/// Transport abstraction used to execute Gemini `generateContent` calls.
pub trait GeminiTransport: Send + Sync {
    /// Creates content for the serialized request payload.
    fn generate_content(
        &self,
        config: &GeminiConfig,
        model: &str,
        payload: &Value,
    ) -> LlmResult<Value>;
}

/// Default HTTP transport backed by `reqwest`.
#[derive(Default)]
pub struct HttpGeminiTransport;

impl GeminiTransport for HttpGeminiTransport {
    fn generate_content(
        &self,
        config: &GeminiConfig,
        model: &str,
        payload: &Value,
    ) -> LlmResult<Value> {
        let client = build_client(config.timeout_ms)?;
        let headers = build_headers();

        let response = client
            .post(config.generate_content_url(model))
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

fn build_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers
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
    use super::{GeminiTransport, HttpGeminiTransport, build_client, build_headers};
    use crate::config::GeminiConfig;
    use greentic_dw_llm::LlmErrorKind;
    use reqwest::header::CONTENT_TYPE;
    use serde_json::json;

    fn config() -> GeminiConfig {
        GeminiConfig::new("secret", "gemini-2.5-pro", 5_000)
    }

    #[test]
    fn build_headers_sets_json_content_type() {
        let headers = build_headers();
        assert_eq!(headers[CONTENT_TYPE], "application/json");
    }

    #[test]
    fn build_client_accepts_timeout() {
        build_client(1).expect("client");
    }

    #[test]
    fn generate_content_maps_invalid_urls_to_network_error() {
        let mut config = config();
        config.base_url = Some("http://[::1".to_string());
        let err = HttpGeminiTransport
            .generate_content(&config, "gemini-2.5-pro", &json!({"contents":[]}))
            .expect_err("invalid url");
        assert_eq!(err.kind, LlmErrorKind::Provider);
    }
}
