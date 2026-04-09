#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Redis-backed short-term memory provider.

use greentic_dw_memory::{
    ShortTermMemory, ShortTermMemoryConfig, ShortTermMemoryPutRequest, StoreBackedShortTermMemory,
};
use greentic_dw_providers_common::{
    ShortTermMemoryVariant, short_term_memory_pack_manifest, short_term_memory_provider_decl,
};
use greentic_state::redis_store::RedisStateStore;
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use serde_json::Value;

/// Redis-backed short-term memory provider.
pub struct RedisShortTermMemory {
    redis_url: String,
    inner: StoreBackedShortTermMemory<RedisStateStore>,
}

impl RedisShortTermMemory {
    /// Creates a provider backed by the supplied Redis URL.
    pub fn from_url(redis_url: impl Into<String>, config: ShortTermMemoryConfig) -> GResult<Self> {
        let redis_url = redis_url.into();
        let store = RedisStateStore::from_url(&redis_url)?;
        Ok(Self {
            redis_url,
            inner: StoreBackedShortTermMemory::new(store, config),
        })
    }

    /// Returns the Redis URL used by this provider.
    #[must_use]
    pub fn redis_url(&self) -> &str {
        &self.redis_url
    }

    /// Returns the active configuration.
    #[must_use]
    pub fn config(&self) -> &ShortTermMemoryConfig {
        self.inner.config()
    }

    /// Returns the canonical provider declaration for this backend.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        short_term_memory_provider_decl(ShortTermMemoryVariant::Redis)
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.memory.short-term.redis")?;
        short_term_memory_pack_manifest(pack_id, ShortTermMemoryVariant::Redis)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }
}

impl ShortTermMemory for RedisShortTermMemory {
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
    use super::RedisShortTermMemory;
    use greentic_dw_memory::ShortTermMemoryConfig;

    #[test]
    fn redis_provider_exposes_shared_manifest() {
        let manifest = RedisShortTermMemory::pack_manifest().expect("manifest");
        assert_eq!(
            manifest.pack_id.as_str(),
            "greentic.dw.providers.memory.short-term.redis"
        );
    }

    #[test]
    fn redis_provider_builds_without_connecting() {
        let provider = RedisShortTermMemory::from_url(
            "redis://127.0.0.1:6379/0",
            ShortTermMemoryConfig::default().with_prefix("dw.memory.test"),
        )
        .expect("provider builds");

        assert_eq!(provider.redis_url(), "redis://127.0.0.1:6379/0");
        assert_eq!(provider.config().prefix, "dw.memory.test");
    }
}
