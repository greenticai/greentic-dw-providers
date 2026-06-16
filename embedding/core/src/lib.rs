#![forbid(unsafe_code)]
//! Shared normalized embedding provider contract for Greentic DW.
//!
//! Mirrors `greentic-dw-llm`: a sync provider trait plus normalized request/
//! response types. Backends implement [`EmbeddingProvider`] over their own
//! transport. The Chronicle doc-RAG adapter (W2) bridges this to its async
//! `EmbedderClient`.

use std::error::Error as StdError;
use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Canonical runtime capability URI for the embedding family.
pub const EMBEDDING_CAPABILITY_URI: &str = "cap://dw.embedding";
/// Canonical pack capability identifier for the embedding family.
pub const EMBEDDING_PACK_CAPABILITY_ID: &str = "greentic.cap.embedding";
/// Default embedding model (matches the Chronicle default).
pub const DEFAULT_EMBEDDING_MODEL: &str = "text-embedding-3-small";
/// Default embedding dimension (matches the Chronicle default).
pub const DEFAULT_EMBEDDING_DIM: usize = 1024;

/// Error categories for embedding operations (mirrors `LlmErrorKind`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EmbeddingErrorKind {
    InvalidRequest,
    Auth,
    Timeout,
    Network,
    RateLimited,
    Provider,
    Internal,
}

/// Normalized embedding error.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingError {
    pub kind: EmbeddingErrorKind,
    pub message: String,
    pub retryable: bool,
}

impl EmbeddingError {
    #[must_use]
    pub fn new(kind: EmbeddingErrorKind, message: impl Into<String>) -> Self {
        Self { kind, message: message.into(), retryable: false }
    }
    #[must_use]
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(EmbeddingErrorKind::InvalidRequest, message)
    }
    #[must_use]
    pub fn provider(message: impl Into<String>) -> Self {
        Self::new(EmbeddingErrorKind::Provider, message)
    }
    #[must_use]
    pub fn retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }
}

impl fmt::Display for EmbeddingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for EmbeddingError {}

/// Result alias for embedding operations.
pub type EmbeddingResult<T> = Result<T, EmbeddingError>;

use greentic_types::TenantCtx;

/// A normalized embedding request. Batch-first: single-input callers pass a
/// one-element `inputs`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    pub request_id: String,
    /// Model override; `None` uses the provider's configured default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub inputs: Vec<String>,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub metadata: Value,
}

impl EmbeddingRequest {
    #[must_use]
    pub fn new(request_id: impl Into<String>, inputs: Vec<String>) -> Self {
        Self { request_id: request_id.into(), model: None, inputs, metadata: Value::Null }
    }
    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}

/// Token accounting, when the provider reports it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u32>,
}

/// A normalized embedding response: one vector per input, order-preserving.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub dim: usize,
    pub vectors: Vec<Vec<f32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<EmbeddingUsage>,
    #[serde(default, skip_serializing_if = "Value::is_null")]
    pub metadata: Value,
}

/// Declared capabilities of an embedding backend.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmbeddingProviderFeatures {
    /// Supports embedding more than one input per call.
    pub batch: bool,
    /// Supports a configurable output dimension.
    pub configurable_dim: bool,
    /// Runs against a local / self-hosted endpoint.
    pub local_self_hosted: bool,
}

impl EmbeddingProviderFeatures {
    #[must_use]
    pub const fn new(batch: bool, configurable_dim: bool, local_self_hosted: bool) -> Self {
        Self { batch, configurable_dim, local_self_hosted }
    }

    #[must_use]
    pub fn operations(&self) -> Vec<String> {
        let mut ops = vec!["embedding.embed".to_string()];
        if self.batch {
            ops.push("embedding.embed_batch".to_string());
        }
        ops
    }

    /// Validates a request against declared features.
    pub fn validate_request(&self, request: &EmbeddingRequest) -> EmbeddingResult<()> {
        if request.inputs.is_empty() {
            return Err(EmbeddingError::invalid_request("inputs must not be empty"));
        }
        if request.inputs.iter().any(|i| i.is_empty()) {
            return Err(EmbeddingError::invalid_request("inputs must not contain empty strings"));
        }
        if request.inputs.len() > 1 && !self.batch {
            return Err(EmbeddingError::invalid_request(
                "provider does not support batch embedding",
            ));
        }
        Ok(())
    }
}

/// Contract implemented by embedding backends. Sync, mirroring `LlmProvider`;
/// the async bridge to Chronicle's `EmbedderClient` lives in W2.
///
/// All methods default to delegating to [`EmbeddingProviderFeatures::validate_request`].
/// Backends override only `features` and `embed`.
pub trait EmbeddingProvider: Send + Sync {
    fn features(&self) -> &EmbeddingProviderFeatures;
    fn embed(&self, tenant: &TenantCtx, request: EmbeddingRequest) -> EmbeddingResult<EmbeddingResponse>;
    fn validate_request(&self, request: &EmbeddingRequest) -> EmbeddingResult<()> {
        self.features().validate_request(request)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn capability_constants_are_stable() {
        assert_eq!(EMBEDDING_CAPABILITY_URI, "cap://dw.embedding");
        assert_eq!(EMBEDDING_PACK_CAPABILITY_ID, "greentic.cap.embedding");
        assert_eq!(DEFAULT_EMBEDDING_DIM, 1024);
    }

    #[test]
    fn error_constructors_set_kind_and_message() {
        let err = EmbeddingError::invalid_request("bad").retryable(true);
        assert_eq!(err.kind, EmbeddingErrorKind::InvalidRequest);
        assert_eq!(err.to_string(), "bad");
        assert!(err.retryable);
    }

    #[test]
    fn features_operations_reflect_flags() {
        let single = EmbeddingProviderFeatures::new(false, true, false);
        assert_eq!(single.operations(), vec!["embedding.embed".to_string()]);
        let batch = EmbeddingProviderFeatures::new(true, true, false);
        assert_eq!(
            batch.operations(),
            vec!["embedding.embed".to_string(), "embedding.embed_batch".to_string()]
        );
    }

    #[test]
    fn validate_request_rejects_empty_inputs() {
        let features = EmbeddingProviderFeatures::new(true, true, false);
        let req = EmbeddingRequest::new("r1", vec![]);
        let err = features.validate_request(&req).expect_err("empty inputs");
        assert_eq!(err.kind, EmbeddingErrorKind::InvalidRequest);
    }

    #[test]
    fn validate_request_rejects_batch_when_unsupported() {
        let features = EmbeddingProviderFeatures::new(false, true, false);
        let req = EmbeddingRequest::new("r1", vec!["a".into(), "b".into()]);
        let err = features.validate_request(&req).expect_err("batch unsupported");
        assert_eq!(err.kind, EmbeddingErrorKind::InvalidRequest);
    }

    #[test]
    fn request_response_roundtrip_serde() {
        let req = EmbeddingRequest::new("r1", vec!["hello".into()]).with_model("m");
        let json = serde_json::to_string(&req).expect("ser");
        let back: EmbeddingRequest = serde_json::from_str(&json).expect("de");
        assert_eq!(back, req);

        let resp = EmbeddingResponse {
            response_id: None,
            model: Some("m".into()),
            dim: 3,
            vectors: vec![vec![0.1, 0.2, 0.3]],
            usage: Some(EmbeddingUsage { prompt_tokens: Some(2), total_tokens: Some(2) }),
            metadata: Value::Null,
        };
        let json = serde_json::to_string(&resp).expect("ser");
        let back: EmbeddingResponse = serde_json::from_str(&json).expect("de");
        assert_eq!(back, resp);
    }
}
