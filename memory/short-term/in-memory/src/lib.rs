#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! In-memory short-term memory provider.

use greentic_dw_memory::{
    ShortTermMemory, ShortTermMemoryConfig, ShortTermMemoryPutRequest, StoreBackedShortTermMemory,
};
use greentic_dw_providers_common::{
    ShortTermMemoryVariant, short_term_memory_pack_manifest, short_term_memory_provider_decl,
};
use greentic_state::inmemory::InMemoryStateStore;
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use serde_json::Value;

/// In-memory short-term memory provider.
pub struct InMemoryShortTermMemory {
    inner: StoreBackedShortTermMemory<InMemoryStateStore>,
}

impl InMemoryShortTermMemory {
    /// Creates a provider with the supplied configuration.
    #[must_use]
    pub fn new(config: ShortTermMemoryConfig) -> Self {
        Self {
            inner: StoreBackedShortTermMemory::new(InMemoryStateStore::new(), config),
        }
    }

    /// Returns the canonical provider declaration for this backend.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        short_term_memory_provider_decl(ShortTermMemoryVariant::InMemory)
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.memory.short-term.in-memory")?;
        short_term_memory_pack_manifest(pack_id, ShortTermMemoryVariant::InMemory)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }

    /// Returns the active configuration.
    #[must_use]
    pub fn config(&self) -> &ShortTermMemoryConfig {
        self.inner.config()
    }
}

impl Default for InMemoryShortTermMemory {
    fn default() -> Self {
        Self::new(ShortTermMemoryConfig::default())
    }
}

impl ShortTermMemory for InMemoryShortTermMemory {
    fn get(&self, tenant: &TenantCtx, key: &str) -> GResult<Option<Value>> {
        self.inner.get(tenant, key)
    }

    fn put(&self, tenant: &TenantCtx, request: ShortTermMemoryPutRequest) -> GResult<()> {
        self.inner.put(tenant, request)
    }

    fn delete(&self, tenant: &TenantCtx, key: &str) -> GResult<bool> {
        self.inner.delete(tenant, key)
    }

    fn clear(&self, tenant: &TenantCtx) -> GResult<u64> {
        self.inner.clear(tenant)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::InMemoryShortTermMemory;
    use greentic_dw_memory::{ShortTermMemory, ShortTermMemoryPutRequest};
    use greentic_types::{EnvId, TenantCtx, TenantId};
    use serde_json::json;

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-memory").expect("tenant id"),
        )
    }

    #[test]
    fn in_memory_provider_exposes_shared_manifest() {
        let manifest = InMemoryShortTermMemory::pack_manifest().expect("manifest");
        assert_eq!(
            manifest.pack_id.as_str(),
            "greentic.dw.providers.memory.short-term.in-memory"
        );
    }

    #[test]
    fn in_memory_provider_roundtrips() {
        let provider = InMemoryShortTermMemory::default();
        let tenant = tenant();

        provider
            .put(
                &tenant,
                ShortTermMemoryPutRequest::new("context", json!({"topic": "dw"})),
            )
            .expect("put succeeds");

        assert_eq!(
            provider.get(&tenant, "context").expect("get succeeds"),
            Some(json!({"topic": "dw"}))
        );
    }
}
