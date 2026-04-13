use std::time::Duration;

use greentic_dw_llm::{LlmError, LlmErrorKind, LlmResult};
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;

use crate::config::OpenAiCompatibleConfig;
use crate::mapping::map_http_error;

const RESERVED_HEADER_NAMES: &[&str] = &["authorization", "content-type"];

/// Transport abstraction used to execute compatible API calls.
pub trait OpenAiCompatibleTransport: Send + Sync {
    /// Executes the serialized request payload.
    fn execute(&self, config: &OpenAiCompatibleConfig, payload: &Value) -> LlmResult<Value>;
}

/// Default HTTP transport backed by `reqwest`.
#[derive(Default)]
pub struct HttpOpenAiCompatibleTransport;

impl OpenAiCompatibleTransport for HttpOpenAiCompatibleTransport {
    fn execute(&self, config: &OpenAiCompatibleConfig, payload: &Value) -> LlmResult<Value> {
        let client = build_client(config.timeout_ms)?;
        let headers = build_headers(config)?;

        let response = client
            .post(config.endpoint_url())
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

fn build_headers(config: &OpenAiCompatibleConfig) -> LlmResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    if let Some(api_key_secret) = &config.api_key_secret {
        let auth_value = HeaderValue::from_str(&format!("Bearer {}", api_key_secret))
            .map_err(|err| LlmError::new(LlmErrorKind::InvalidRequest, err.to_string()))?;
        headers.insert(AUTHORIZATION, auth_value);
    }
    for (name, value) in &config.headers {
        if let Some(value) = value.as_str() {
            if RESERVED_HEADER_NAMES.contains(&name.to_ascii_lowercase().as_str()) {
                return Err(LlmError::new(
                    LlmErrorKind::InvalidRequest,
                    format!("custom header '{name}' is reserved"),
                ));
            }
            let header_name = HeaderName::from_bytes(name.as_bytes())
                .map_err(|err| LlmError::new(LlmErrorKind::InvalidRequest, err.to_string()))?;
            let header_value = HeaderValue::from_str(value)
                .map_err(|err| LlmError::new(LlmErrorKind::InvalidRequest, err.to_string()))?;
            headers.insert(header_name, header_value);
        }
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
    use super::{
        HttpOpenAiCompatibleTransport, OpenAiCompatibleTransport, build_client, build_headers,
    };
    use crate::config::OpenAiCompatibleConfig;
    use greentic_dw_llm::LlmErrorKind;
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
    use serde_json::json;

    fn config() -> OpenAiCompatibleConfig {
        let mut config =
            OpenAiCompatibleConfig::new("https://gateway.example/v1", "model-x", 5_000);
        config.api_key_secret = Some("secret".to_string());
        config
            .headers
            .insert("x-extra".to_string(), json!("present"));
        config
    }

    #[test]
    fn build_headers_includes_auth_and_extra_headers() {
        let headers = build_headers(&config()).expect("headers");
        assert_eq!(headers[AUTHORIZATION], "Bearer secret");
        assert_eq!(headers[CONTENT_TYPE], "application/json");
        assert_eq!(headers["x-extra"], "present");
    }

    #[test]
    fn build_headers_rejects_invalid_header_names() {
        let mut config = config();
        config.headers.insert("bad header".to_string(), json!("x"));
        let err = build_headers(&config).expect_err("invalid header");
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
    }

    #[test]
    fn build_headers_rejects_reserved_header_override() {
        let mut config = config();
        config
            .headers
            .insert("authorization".to_string(), json!("Bearer attacker"));
        let err = build_headers(&config).expect_err("reserved header");
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
        assert!(err.message.contains("reserved"));
    }

    #[test]
    fn build_client_accepts_timeout() {
        build_client(1).expect("client");
    }

    #[test]
    fn execute_fails_before_network_for_invalid_auth_header() {
        let mut config = config();
        config.api_key_secret = Some("bad\nkey".to_string());
        let err = HttpOpenAiCompatibleTransport
            .execute(&config, &json!({"input":"hello"}))
            .expect_err("invalid header");
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
    }
}
