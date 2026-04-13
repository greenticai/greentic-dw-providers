use std::time::Duration;

use greentic_dw_llm::{LlmError, LlmErrorKind, LlmResult};
use reqwest::blocking::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use serde_json::Value;

use crate::config::{AzureOpenAiAuthMode, AzureOpenAiConfig};
use crate::mapping::map_http_error;

const RESERVED_HEADER_NAMES: &[&str] = &["authorization", "content-type", "api-key"];

/// Transport abstraction used to execute Azure OpenAI calls.
pub trait AzureOpenAiTransport: Send + Sync {
    /// Executes the serialized request payload.
    fn execute(&self, config: &AzureOpenAiConfig, payload: &Value) -> LlmResult<Value>;
}

/// Default HTTP transport backed by `reqwest`.
#[derive(Default)]
pub struct HttpAzureOpenAiTransport;

impl AzureOpenAiTransport for HttpAzureOpenAiTransport {
    fn execute(&self, config: &AzureOpenAiConfig, payload: &Value) -> LlmResult<Value> {
        let client = build_client(config.timeout_ms)?;
        let headers = build_headers(config)?;

        let url = if config.use_responses_api {
            config.responses_url()
        } else {
            config.chat_completions_url()
        };

        let response = client
            .post(url)
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

fn build_headers(config: &AzureOpenAiConfig) -> LlmResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    match config.auth_mode {
        AzureOpenAiAuthMode::ApiKey => {
            let api_key = config.api_key_secret.as_deref().unwrap_or_default();
            headers.insert(
                "api-key",
                HeaderValue::from_str(api_key)
                    .map_err(|err| LlmError::new(LlmErrorKind::InvalidRequest, err.to_string()))?,
            );
        }
        AzureOpenAiAuthMode::EntraId => {
            let token = config.entra_token_secret.as_deref().unwrap_or_default();
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {token}"))
                    .map_err(|err| LlmError::new(LlmErrorKind::InvalidRequest, err.to_string()))?,
            );
        }
    }

    for (name, value) in &config.raw_headers {
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
    use super::{AzureOpenAiTransport, HttpAzureOpenAiTransport, build_client, build_headers};
    use crate::config::{AzureOpenAiAuthMode, AzureOpenAiConfig};
    use greentic_dw_llm::LlmErrorKind;
    use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
    use serde_json::json;

    fn api_key_config() -> AzureOpenAiConfig {
        let mut config = AzureOpenAiConfig::new("https://example.openai.azure.com", "gpt-5", 5_000);
        config.api_key_secret = Some("secret".to_string());
        config
            .raw_headers
            .insert("x-extra".to_string(), json!("present"));
        config
    }

    #[test]
    fn build_headers_includes_api_key_and_extra_headers() {
        let headers = build_headers(&api_key_config()).expect("headers");
        assert_eq!(headers[CONTENT_TYPE], "application/json");
        assert_eq!(headers["api-key"], "secret");
        assert_eq!(headers["x-extra"], "present");
    }

    #[test]
    fn build_headers_supports_entra_id_auth() {
        let mut config = api_key_config();
        config.auth_mode = AzureOpenAiAuthMode::EntraId;
        config.entra_token_secret = Some("token".to_string());
        let headers = build_headers(&config).expect("headers");
        assert_eq!(headers[AUTHORIZATION], "Bearer token");
    }

    #[test]
    fn build_headers_rejects_invalid_header_values() {
        let mut config = api_key_config();
        config
            .raw_headers
            .insert("x-extra".to_string(), json!("bad\nvalue"));
        let err = build_headers(&config).expect_err("invalid header");
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
    }

    #[test]
    fn build_headers_rejects_reserved_header_override() {
        let mut config = api_key_config();
        config
            .raw_headers
            .insert("api-key".to_string(), json!("attacker"));
        let err = build_headers(&config).expect_err("reserved header");
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
        assert!(err.message.contains("reserved"));
    }

    #[test]
    fn build_client_accepts_timeout() {
        build_client(1).expect("client");
    }

    #[test]
    fn execute_fails_before_network_for_invalid_api_key_header() {
        let mut config = api_key_config();
        config.api_key_secret = Some("bad\nkey".to_string());
        let err = HttpAzureOpenAiTransport
            .execute(&config, &json!({"input":"hello"}))
            .expect_err("invalid header");
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
    }
}
