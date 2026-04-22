#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Deterministic static context provider.

use greentic_dw_context::{
    ContextFragment, ContextPackage, ContextProvider, ContextRequest, ContextSourceKind,
};
use greentic_types::{GResult, TenantCtx};
use serde_json::{Map, Value};

/// Static context provider configuration.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StaticContextConfig {
    /// Preconfigured fragments returned first in deterministic order.
    pub fragments: Vec<ContextFragment>,
    /// Whether to append runtime metadata as a fragment.
    pub include_runtime_metadata: bool,
}

/// Deterministic static context provider.
pub struct StaticContextProvider {
    config: StaticContextConfig,
}

impl StaticContextProvider {
    /// Creates a static context provider.
    #[must_use]
    pub fn new(config: StaticContextConfig) -> Self {
        Self { config }
    }
}

impl ContextProvider for StaticContextProvider {
    fn assemble(&self, _tenant: &TenantCtx, request: ContextRequest) -> GResult<ContextPackage> {
        let mut package = ContextPackage::empty();
        for fragment in &self.config.fragments {
            package = package.with_fragment(fragment.clone());
        }
        if self.config.include_runtime_metadata
            && request.runtime_metadata != Value::Object(Map::new())
        {
            package = package.with_fragment(
                ContextFragment::new("runtime-metadata", request.runtime_metadata.clone())?
                    .with_source_kind(ContextSourceKind::Static)
                    .with_provenance_ref(format!("request:{}", request.request_id))
                    .with_score(1.0)
                    .with_pinned(true),
            );
        }
        let fragment_count = package.fragments.len();
        Ok(package.with_metadata(serde_json::json!({
            "provider": "static",
            "fragment_count": fragment_count
        })))
    }
}
