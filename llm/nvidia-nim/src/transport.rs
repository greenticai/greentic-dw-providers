use std::time::Duration;

use greentic_dw_llm::{LlmError, LlmErrorKind, LlmResult};
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;

use crate::config::NvidiaNimConfig;
use crate::mapping::map_http_error;

const RESERVED_HEADER_NAMES: &[&str] = &["authorization", "content-type"];

/// Transport abstraction used to execute NIM requests.
pub trait NvidiaNimTransport: Send + Sync {
    /// Executes an inference request.
    fn infer(&self, config: &NvidiaNimConfig, payload: &Value) -> LlmResult<Value>;
    /// Fetches the model listing.
    fn models(&self, config: &NvidiaNimConfig) -> LlmResult<Value>;
    /// Fetches ready health information.
    fn ready(&self, config: &NvidiaNimConfig) -> LlmResult<Value>;
    /// Fetches live health information.
    fn live(&self, config: &NvidiaNimConfig) -> LlmResult<Value>;
}

/// Default HTTP transport backed by `reqwest`.
#[derive(Default)]
pub struct HttpNvidiaNimTransport;

impl NvidiaNimTransport for HttpNvidiaNimTransport {
    fn infer(&self, config: &NvidiaNimConfig, payload: &Value) -> LlmResult<Value> {
        execute_post(config, &config.inference_url(), payload, "inference")
    }

    fn models(&self, config: &NvidiaNimConfig) -> LlmResult<Value> {
        execute_get(config, &config.models_url(), "discovery")
    }

    fn ready(&self, config: &NvidiaNimConfig) -> LlmResult<Value> {
        execute_get(config, &config.ready_url(), "health")
    }

    fn live(&self, config: &NvidiaNimConfig) -> LlmResult<Value> {
        execute_get(config, &config.live_url(), "health")
    }
}

fn client(config: &NvidiaNimConfig) -> LlmResult<Client> {
    Client::builder()
        .timeout(Duration::from_millis(config.timeout_ms))
        .build()
        .map_err(|err| LlmError::new(LlmErrorKind::Internal, err.to_string()))
}

fn headers(config: &NvidiaNimConfig) -> LlmResult<HeaderMap> {
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

fn execute_post(
    config: &NvidiaNimConfig,
    url: &str,
    payload: &Value,
    endpoint: &'static str,
) -> LlmResult<Value> {
    let response = client(config)?
        .post(url)
        .headers(headers(config)?)
        .json(payload)
        .send()
        .map_err(map_reqwest_error)?;
    let status = response.status();
    let body = response
        .text()
        .map_err(|err| LlmError::new(LlmErrorKind::Network, err.to_string()))?;
    if !status.is_success() {
        return Err(map_http_error(status.as_u16(), &body, endpoint));
    }
    serde_json::from_str(&body)
        .map_err(|err| LlmError::new(LlmErrorKind::Provider, err.to_string()))
}

fn execute_get(config: &NvidiaNimConfig, url: &str, endpoint: &'static str) -> LlmResult<Value> {
    let response = client(config)?
        .get(url)
        .headers(headers(config)?)
        .send()
        .map_err(map_reqwest_error)?;
    let status = response.status();
    let body = response
        .text()
        .map_err(|err| LlmError::new(LlmErrorKind::Network, err.to_string()))?;
    if !status.is_success() {
        return Err(map_http_error(status.as_u16(), &body, endpoint));
    }
    serde_json::from_str(&body)
        .map_err(|err| LlmError::new(LlmErrorKind::Provider, err.to_string()))
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
    use super::{HttpNvidiaNimTransport, NvidiaNimTransport, client, headers};
    use crate::config::NvidiaNimConfig;
    use greentic_dw_llm::LlmErrorKind;
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
    use serde_json::json;

    fn config() -> NvidiaNimConfig {
        let mut config = NvidiaNimConfig::new("https://nim.example/v1", "meta/llama", 5_000);
        config.api_key_secret = Some("secret".to_string());
        config
            .headers
            .insert("x-extra".to_string(), json!("present"));
        config
    }

    #[test]
    fn headers_include_auth_and_extra_headers() {
        let built = headers(&config()).expect("headers");
        assert_eq!(built[AUTHORIZATION], "Bearer secret");
        assert_eq!(built[CONTENT_TYPE], "application/json");
        assert_eq!(built["x-extra"], "present");
    }

    #[test]
    fn headers_reject_invalid_names() {
        let mut config = config();
        config
            .headers
            .insert("bad header".to_string(), json!("value"));
        let err = headers(&config).expect_err("invalid header");
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
    }

    #[test]
    fn headers_reject_reserved_header_override() {
        let mut config = config();
        config
            .headers
            .insert("authorization".to_string(), json!("Bearer attacker"));
        let err = headers(&config).expect_err("reserved header");
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
        assert!(err.message.contains("reserved"));
    }

    #[test]
    fn client_accepts_timeout() {
        client(&config()).expect("client");
    }

    #[test]
    fn infer_fails_before_network_for_invalid_auth_header() {
        let mut config = config();
        config.api_key_secret = Some("bad\nkey".to_string());
        let err = HttpNvidiaNimTransport
            .infer(&config, &json!({"messages":[]}))
            .expect_err("invalid header");
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
    }
}
