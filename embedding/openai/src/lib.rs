#![forbid(unsafe_code)]
//! Native OpenAI embedding provider for Greentic DW.

mod config;
mod errors;
mod mapping;
mod transport;

pub use config::{OpenAiEmbeddingConfig, OPENAI_DEFAULT_BASE_URL};
pub use errors::OpenAiEmbeddingError;
pub use transport::{HttpOpenAiEmbeddingTransport, OpenAiEmbeddingTransport};

use greentic_dw_embedding::{
    EmbeddingProvider, EmbeddingProviderFeatures, EmbeddingRequest, EmbeddingResponse,
    EmbeddingResult,
};
use greentic_types::TenantCtx;

use crate::mapping::{build_embeddings_request, parse_embeddings_response};

/// OpenAI embedding provider over a pluggable transport.
pub struct OpenAiEmbeddingProvider<T = HttpOpenAiEmbeddingTransport> {
    config: OpenAiEmbeddingConfig,
    features: EmbeddingProviderFeatures,
    transport: T,
}

impl OpenAiEmbeddingProvider<HttpOpenAiEmbeddingTransport> {
    pub fn new(config: OpenAiEmbeddingConfig) -> EmbeddingResult<Self> {
        Self::with_transport(config, HttpOpenAiEmbeddingTransport)
    }
}

impl<T> OpenAiEmbeddingProvider<T>
where
    T: OpenAiEmbeddingTransport,
{
    pub fn with_transport(config: OpenAiEmbeddingConfig, transport: T) -> EmbeddingResult<Self> {
        config.validate().map_err(greentic_dw_embedding::EmbeddingError::from)?;
        // batch supported, dim configurable, not local.
        let features = EmbeddingProviderFeatures::new(true, true, false);
        Ok(Self { config, features, transport })
    }

    #[must_use]
    pub fn config(&self) -> &OpenAiEmbeddingConfig {
        &self.config
    }
}

impl<T> EmbeddingProvider for OpenAiEmbeddingProvider<T>
where
    T: OpenAiEmbeddingTransport,
{
    fn features(&self) -> &EmbeddingProviderFeatures {
        &self.features
    }

    fn embed(&self, _tenant: &TenantCtx, request: EmbeddingRequest) -> EmbeddingResult<EmbeddingResponse> {
        self.validate_request(&request)?;
        let payload = build_embeddings_request(&self.config, &request);
        let raw = self.transport.create_embeddings(&self.config, &payload)?;
        parse_embeddings_response(&self.config, &raw)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use greentic_types::{EnvId, TenantCtx, TenantId};
    use serde_json::{Value, json};
    use std::sync::Mutex;

    struct RecordingTransport {
        payload: Mutex<Option<Value>>,
        response: Value,
    }
    impl RecordingTransport {
        fn new(response: Value) -> Self {
            Self { payload: Mutex::new(None), response }
        }
        fn payload(&self) -> Value {
            self.payload.lock().expect("payload mutex").clone().expect("captured payload")
        }
    }
    impl OpenAiEmbeddingTransport for RecordingTransport {
        fn create_embeddings(&self, _config: &OpenAiEmbeddingConfig, payload: &Value) -> EmbeddingResult<Value> {
            *self.payload.lock().expect("payload mutex") = Some(payload.clone());
            Ok(self.response.clone())
        }
    }

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-embed").expect("tenant id"),
        )
    }
    fn config() -> OpenAiEmbeddingConfig {
        let mut c = OpenAiEmbeddingConfig::with_defaults("sk-test", 10_000);
        c.embedding_dim = 3;
        c
    }

    #[test]
    fn request_maps_to_model_and_input() {
        let transport = RecordingTransport::new(json!({"data": [], "model": "text-embedding-3-small"}));
        let provider = OpenAiEmbeddingProvider::with_transport(config(), transport).expect("provider");
        let req = EmbeddingRequest::new("r1", vec!["alpha".into(), "beta".into()]);
        let _ = provider.embed(&tenant(), req).expect("embed");
        let payload = provider_payload(&provider);
        assert_eq!(payload["model"], "text-embedding-3-small");
        assert_eq!(payload["input"], json!(["alpha", "beta"]));
    }

    // helper to read the recording transport back out
    fn provider_payload(p: &OpenAiEmbeddingProvider<RecordingTransport>) -> Value {
        p.transport.payload()
    }

    #[test]
    fn response_parses_in_index_order_and_truncates() {
        let transport = RecordingTransport::new(json!({
            "model": "text-embedding-3-small",
            "data": [
                {"index": 1, "embedding": [0.4, 0.5, 0.6, 0.7]},
                {"index": 0, "embedding": [0.1, 0.2, 0.3, 0.9]}
            ],
            "usage": {"prompt_tokens": 4, "total_tokens": 4}
        }));
        let provider = OpenAiEmbeddingProvider::with_transport(config(), transport).expect("provider");
        let req = EmbeddingRequest::new("r1", vec!["a".into(), "b".into()]);
        let resp = provider.embed(&tenant(), req).expect("embed");
        assert_eq!(resp.dim, 3);
        // index 0 first; each truncated to dim=3 (no padding)
        assert_eq!(resp.vectors, vec![vec![0.1, 0.2, 0.3], vec![0.4, 0.5, 0.6]]);
        assert_eq!(resp.usage.expect("usage").total_tokens, Some(4));
    }

    #[test]
    fn empty_inputs_rejected_before_transport() {
        let transport = RecordingTransport::new(json!({"data": []}));
        let provider = OpenAiEmbeddingProvider::with_transport(config(), transport).expect("provider");
        let req = EmbeddingRequest::new("r1", vec![]);
        assert!(provider.embed(&tenant(), req).is_err());
    }
}
