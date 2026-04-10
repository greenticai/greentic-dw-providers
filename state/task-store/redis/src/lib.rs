#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Redis-backed task-store provider.

use greentic_dw_providers_common::{
    TaskStoreVariant, task_store_pack_manifest, task_store_provider_decl,
};
use greentic_dw_task_store::{StoreBackedTaskStore, TaskRecord, TaskStore, TaskStoreConfig};
use greentic_state::redis_store::RedisStateStore;
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};

/// Redis-backed task-store provider.
pub struct RedisTaskStore {
    redis_url: String,
    inner: StoreBackedTaskStore<RedisStateStore>,
}

impl RedisTaskStore {
    /// Creates a provider backed by the supplied Redis URL.
    pub fn from_url(redis_url: impl Into<String>, config: TaskStoreConfig) -> GResult<Self> {
        let redis_url = redis_url.into();
        let store = RedisStateStore::from_url(&redis_url)?;
        Ok(Self {
            redis_url,
            inner: StoreBackedTaskStore::new(store, config),
        })
    }

    /// Returns the Redis URL used by this provider.
    #[must_use]
    pub fn redis_url(&self) -> &str {
        &self.redis_url
    }

    /// Returns the canonical provider declaration for this backend.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        task_store_provider_decl(TaskStoreVariant::Redis)
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.state.task-store.redis")?;
        task_store_pack_manifest(pack_id, TaskStoreVariant::Redis)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }
}

impl TaskStore for RedisTaskStore {
    fn load(&self, tenant: &TenantCtx, task_id: &str) -> GResult<Option<TaskRecord>> {
        self.inner.load(tenant, task_id)
    }

    fn save(&self, tenant: &TenantCtx, record: TaskRecord) -> GResult<()> {
        self.inner.save(tenant, record)
    }

    fn list(&self, tenant: &TenantCtx) -> GResult<Vec<TaskRecord>> {
        self.inner.list(tenant)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::RedisTaskStore;
    use greentic_dw_task_store::TaskStoreConfig;

    #[test]
    fn redis_task_store_builds_without_connecting() {
        let store = RedisTaskStore::from_url(
            "redis://127.0.0.1:6379/0",
            TaskStoreConfig::default().with_prefix("dw.state.test"),
        )
        .expect("store builds");

        assert_eq!(store.redis_url(), "redis://127.0.0.1:6379/0");
    }

    #[test]
    fn redis_task_store_exposes_shared_provider_decl() {
        assert_eq!(
            RedisTaskStore::provider_decl().provider_type,
            "dw.state.task-store.redis"
        );
    }

    #[test]
    fn redis_task_store_exposes_shared_manifest() {
        let manifest = RedisTaskStore::pack_manifest().expect("manifest");
        assert_eq!(
            manifest.pack_id.as_str(),
            "greentic.dw.providers.state.task-store.redis"
        );
    }
}
