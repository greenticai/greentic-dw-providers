#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! In-memory task-store provider.

use greentic_dw_providers_common::{
    TaskStoreVariant, task_store_pack_manifest, task_store_provider_decl,
};
use greentic_dw_task_store::{StoreBackedTaskStore, TaskRecord, TaskStore, TaskStoreConfig};
use greentic_state::inmemory::InMemoryStateStore;
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};

/// In-memory task-store provider.
pub struct InMemoryTaskStore {
    inner: StoreBackedTaskStore<InMemoryStateStore>,
}

impl InMemoryTaskStore {
    /// Creates a provider with the supplied configuration.
    #[must_use]
    pub fn new(config: TaskStoreConfig) -> Self {
        Self {
            inner: StoreBackedTaskStore::new(InMemoryStateStore::new(), config),
        }
    }

    /// Returns the canonical provider declaration for this backend.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        task_store_provider_decl(TaskStoreVariant::InMemory)
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.state.task-store.in-memory")?;
        task_store_pack_manifest(pack_id, TaskStoreVariant::InMemory)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }
}

impl Default for InMemoryTaskStore {
    fn default() -> Self {
        Self::new(TaskStoreConfig::default())
    }
}

impl TaskStore for InMemoryTaskStore {
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
    use super::InMemoryTaskStore;
    use greentic_dw_task_store::{TaskRecord, TaskStore};
    use greentic_types::{EnvId, TenantCtx, TenantId};
    use serde_json::json;

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-state").expect("tenant id"),
        )
    }

    #[test]
    fn in_memory_task_store_roundtrips() {
        let store = InMemoryTaskStore::default();
        let tenant = tenant();
        store
            .save(
                &tenant,
                TaskRecord::new("task-1", json!({"status": "running"})).expect("record"),
            )
            .expect("save");

        assert_eq!(store.list(&tenant).expect("list").len(), 1);
    }
}
