use std::time::Duration;

use greentic_dw_llm::{LlmError, LlmErrorKind, LlmResult};
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde_json::Value;

use crate::config::OpenAiConfig;
use crate::mapping::map_http_error;

/// Transport abstraction used to execute OpenAI Responses API calls.
pub trait OpenAiTransport: Send + Sync {
    /// Creates a response for the serialized request payload.
    fn create_response(&self, config: &OpenAiConfig, payload: &Value) -> LlmResult<Value>;
}

/// Default HTTP transport backed by `reqwest`.
#[derive(Default)]
pub struct HttpOpenAiTransport;

impl OpenAiTransport for HttpOpenAiTransport {
    fn create_response(&self, config: &OpenAiConfig, payload: &Value) -> LlmResult<Value> {
        let client = build_client(config.timeout_ms)?;
        let headers = build_headers(config)?;

        let response = client
            .post(config.responses_url())
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

fn build_headers(config: &OpenAiConfig) -> LlmResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    let auth_value = HeaderValue::from_str(&format!("Bearer {}", config.api_key_secret))
        .map_err(|err| LlmError::new(LlmErrorKind::InvalidRequest, err.to_string()))?;
    headers.insert(AUTHORIZATION, auth_value);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    if let Some(organization) = &config.organization {
        headers.insert(
            "OpenAI-Organization",
            HeaderValue::from_str(organization)
                .map_err(|err| LlmError::new(LlmErrorKind::InvalidRequest, err.to_string()))?,
        );
    }
    if let Some(project) = &config.project {
        headers.insert(
            "OpenAI-Project",
            HeaderValue::from_str(project)
                .map_err(|err| LlmError::new(LlmErrorKind::InvalidRequest, err.to_string()))?,
        );
    }

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
    use super::{HttpOpenAiTransport, OpenAiTransport, build_client, build_headers};
    use crate::config::OpenAiConfig;
    use greentic_dw_llm::LlmErrorKind;
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
    use serde_json::json;

    fn config() -> OpenAiConfig {
        let mut config = OpenAiConfig::new("secret", "gpt-5.4", 5_000);
        config.organization = Some("org_123".to_string());
        config.project = Some("proj_456".to_string());
        config
    }

    #[test]
    fn build_headers_includes_auth_and_optional_headers() {
        let headers = build_headers(&config()).expect("headers");
        assert_eq!(headers[AUTHORIZATION], "Bearer secret");
        assert_eq!(headers[CONTENT_TYPE], "application/json");
        assert_eq!(headers["OpenAI-Organization"], "org_123");
        assert_eq!(headers["OpenAI-Project"], "proj_456");
    }

    #[test]
    fn build_headers_rejects_invalid_values() {
        let mut config = config();
        config.organization = Some("bad\nvalue".to_string());
        let err = build_headers(&config).expect_err("invalid header");
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
    }

    #[test]
    fn build_client_accepts_timeout() {
        build_client(1).expect("client");
    }

    #[test]
    fn create_response_fails_before_network_for_invalid_api_key_header() {
        let mut config = config();
        config.api_key_secret = "bad\nkey".to_string();
        let err = HttpOpenAiTransport
            .create_response(&config, &json!({"input":"hello"}))
            .expect_err("invalid header");
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
    }
}
