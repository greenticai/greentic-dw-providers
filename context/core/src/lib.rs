#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Shared context assembly contract and fragment models.

use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// High-level origin for a context fragment.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextSourceKind {
    /// Static configuration or prompt fragment.
    Static,
    /// Workspace artifact materialized from a previous step.
    Workspace,
    /// Plan step metadata or payload.
    PlanStep,
    /// Memory or external reference pointer.
    MemoryRef,
    /// Compression summary produced by another provider.
    Summary,
}

/// A single context fragment with provenance metadata.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContextFragment {
    /// Stable fragment identifier.
    pub id: String,
    /// Structured content for the fragment.
    pub content: Value,
    /// Provenance reference string.
    pub provenance_ref: String,
    /// Source kind for the fragment.
    pub source_kind: ContextSourceKind,
    /// Ranking score used by compression and retrieval.
    pub score: f64,
    /// Whether the fragment is pinned and must not be dropped.
    pub pinned: bool,
}

impl ContextFragment {
    /// Creates a new context fragment.
    pub fn new(id: impl Into<String>, content: Value) -> GResult<Self> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(invalid("context fragment id must not be empty"));
        }
        Ok(Self {
            id,
            content,
            provenance_ref: String::new(),
            source_kind: ContextSourceKind::Static,
            score: 0.0,
            pinned: false,
        })
    }

    /// Sets the provenance reference for the fragment.
    #[must_use]
    pub fn with_provenance_ref(mut self, provenance_ref: impl Into<String>) -> Self {
        self.provenance_ref = provenance_ref.into();
        self
    }

    /// Sets the source kind for the fragment.
    #[must_use]
    pub fn with_source_kind(mut self, source_kind: ContextSourceKind) -> Self {
        self.source_kind = source_kind;
        self
    }

    /// Sets the ranking score for the fragment.
    #[must_use]
    pub fn with_score(mut self, score: f64) -> Self {
        self.score = score;
        self
    }

    /// Pins the fragment in the assembled package.
    #[must_use]
    pub fn with_pinned(mut self, pinned: bool) -> Self {
        self.pinned = pinned;
        self
    }
}

/// A fully assembled context package.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContextPackage {
    /// Ordered context fragments.
    pub fragments: Vec<ContextFragment>,
    /// Optional assembly metadata.
    pub metadata: Value,
}

impl ContextPackage {
    /// Creates an empty context package.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            fragments: Vec::new(),
            metadata: Value::Object(Map::new()),
        }
    }

    /// Appends a fragment to the package.
    #[must_use]
    pub fn with_fragment(mut self, fragment: ContextFragment) -> Self {
        self.fragments.push(fragment);
        self
    }

    /// Sets package metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Request passed to a context provider.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContextRequest {
    /// Stable request identifier.
    pub request_id: String,
    /// Optional input hints for assembly.
    pub hints: Value,
    /// Optional free-form runtime metadata.
    pub runtime_metadata: Value,
}

impl ContextRequest {
    /// Creates a new context request.
    pub fn new(request_id: impl Into<String>) -> GResult<Self> {
        let request_id = request_id.into();
        if request_id.trim().is_empty() {
            return Err(invalid("context request id must not be empty"));
        }
        Ok(Self {
            request_id,
            hints: Value::Null,
            runtime_metadata: Value::Object(Map::new()),
        })
    }

    /// Attaches runtime metadata to the assembly request.
    #[must_use]
    pub fn with_runtime_metadata(mut self, runtime_metadata: Value) -> Self {
        self.runtime_metadata = runtime_metadata;
        self
    }
}

/// Contract implemented by context backends.
pub trait ContextProvider: Send + Sync {
    /// Assembles a context package for a request.
    fn assemble(&self, tenant: &TenantCtx, request: ContextRequest) -> GResult<ContextPackage>;
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{ContextFragment, ContextPackage, ContextRequest};
    use serde_json::json;

    #[test]
    fn request_requires_id() {
        assert!(ContextRequest::new("").is_err());
    }

    #[test]
    fn package_preserves_fragment_order() {
        let package = ContextPackage::empty()
            .with_fragment(ContextFragment::new("a", json!({"rank": 1})).expect("a"))
            .with_fragment(ContextFragment::new("b", json!({"rank": 2})).expect("b"));

        assert_eq!(package.fragments.len(), 2);
        assert_eq!(package.fragments[0].id, "a");
        assert_eq!(package.fragments[1].id, "b");
    }
}
