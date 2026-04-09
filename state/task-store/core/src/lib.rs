#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Shared task-store contract and generic store-backed implementation.

use greentic_dw_providers_common::{task_store_checkpoint_key, task_store_resume_lookup_key};
use greentic_state::StateStore;
use greentic_types::{ErrorCode, GResult, GreenticError, StateKey, TenantCtx};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Default prefix used to scope task-store entries in the shared state store.
pub const DEFAULT_TASK_STORE_PREFIX: &str = "dw.state.task-store";

/// Configuration shared by task-store backends.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskStoreConfig {
    /// Store prefix used to isolate this task-store family.
    pub prefix: String,
}

impl Default for TaskStoreConfig {
    fn default() -> Self {
        Self {
            prefix: DEFAULT_TASK_STORE_PREFIX.to_string(),
        }
    }
}

impl TaskStoreConfig {
    /// Returns a copy with a custom store prefix.
    #[must_use]
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }
}

/// Task-state record stored by the task-store backend.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskRecord {
    /// Stable task identifier.
    pub task_id: String,
    /// Arbitrary state payload for the task.
    pub state: Value,
    /// Optional resume token or checkpoint marker.
    pub resume_token: Option<String>,
    /// Whether the task is currently blocked on an await point.
    pub pending_await: bool,
    /// Last update timestamp in unix milliseconds.
    pub updated_at_ms: u64,
    /// Free-form task metadata.
    pub metadata: BTreeMap<String, String>,
}

impl TaskRecord {
    /// Creates a new task record stamped with the current wall-clock time.
    pub fn new(task_id: impl Into<String>, state: Value) -> GResult<Self> {
        let task_id = task_id.into();
        validate_task_id(&task_id)?;
        Ok(Self {
            task_id,
            state,
            resume_token: None,
            pending_await: false,
            updated_at_ms: now_unix_ms()?,
            metadata: BTreeMap::new(),
        })
    }

    /// Sets a resume token for the record.
    #[must_use]
    pub fn with_resume_token(mut self, token: impl Into<String>) -> Self {
        self.resume_token = Some(token.into());
        self
    }

    /// Marks the record as pending await.
    #[must_use]
    pub fn with_pending_await(mut self, pending_await: bool) -> Self {
        self.pending_await = pending_await;
        self
    }

    /// Adds metadata to the record.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Contract implemented by task-store providers.
pub trait TaskStore: Send + Sync {
    /// Loads a task record by task id.
    fn load(&self, tenant: &TenantCtx, task_id: &str) -> GResult<Option<TaskRecord>>;

    /// Saves or updates a task record.
    fn save(&self, tenant: &TenantCtx, record: TaskRecord) -> GResult<()>;

    /// Lists all known task records for the given tenant.
    fn list(&self, tenant: &TenantCtx) -> GResult<Vec<TaskRecord>>;
}

/// Generic task-store implementation backed by a [`StateStore`].
pub struct StoreBackedTaskStore<S> {
    store: S,
    config: TaskStoreConfig,
}

impl<S> StoreBackedTaskStore<S> {
    /// Creates a new store-backed task-store facade.
    #[must_use]
    pub fn new(store: S, config: TaskStoreConfig) -> Self {
        Self { store, config }
    }

    /// Returns the active configuration.
    #[must_use]
    pub fn config(&self) -> &TaskStoreConfig {
        &self.config
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
struct TaskIndex {
    task_ids: Vec<String>,
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}

fn now_unix_ms() -> GResult<u64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?;
    Ok(duration.as_millis() as u64)
}

fn validate_task_id(task_id: &str) -> GResult<()> {
    if task_id.trim().is_empty() {
        return Err(invalid("task id must not be empty"));
    }
    Ok(())
}

fn index_key() -> GResult<StateKey> {
    Ok(StateKey::new("dw:state:index:task-store"))
}

fn decode_task_index(value: Option<Value>) -> GResult<TaskIndex> {
    match value {
        Some(value) => serde_json::from_value(value)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string())),
        None => Ok(TaskIndex::default()),
    }
}

fn encode_value<T: Serialize>(value: &T) -> GResult<Value> {
    serde_json::to_value(value)
        .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
}

fn decode_task_record(value: Value) -> GResult<TaskRecord> {
    serde_json::from_value(value)
        .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
}

impl<S> StoreBackedTaskStore<S>
where
    S: StateStore,
{
    fn load_index(&self, tenant: &TenantCtx) -> GResult<TaskIndex> {
        let key = index_key()?;
        let value = self
            .store
            .get_json(tenant, &self.config.prefix, &key, None)?;
        decode_task_index(value)
    }

    fn save_index(&self, tenant: &TenantCtx, index: &TaskIndex) -> GResult<()> {
        let key = index_key()?;
        let value = encode_value(index)?;
        self.store
            .set_json(tenant, &self.config.prefix, &key, None, &value, None)
    }
}

impl<S> TaskStore for StoreBackedTaskStore<S>
where
    S: StateStore,
{
    fn load(&self, tenant: &TenantCtx, task_id: &str) -> GResult<Option<TaskRecord>> {
        validate_task_id(task_id)?;
        let key = task_store_checkpoint_key(task_id);
        let value = self
            .store
            .get_json(tenant, &self.config.prefix, &key, None)?;
        value.map(decode_task_record).transpose()
    }

    fn save(&self, tenant: &TenantCtx, mut record: TaskRecord) -> GResult<()> {
        validate_task_id(&record.task_id)?;
        record.updated_at_ms = now_unix_ms()?;

        let checkpoint_key = task_store_checkpoint_key(&record.task_id);
        let checkpoint_value = encode_value(&record)?;
        self.store.set_json(
            tenant,
            &self.config.prefix,
            &checkpoint_key,
            None,
            &checkpoint_value,
            None,
        )?;

        if let Some(token) = &record.resume_token {
            let lookup_key = task_store_resume_lookup_key(&record.task_id);
            let lookup_value = encode_value(&serde_json::json!({
                "task_id": record.task_id,
                "resume_token": token,
            }))?;
            self.store.set_json(
                tenant,
                &self.config.prefix,
                &lookup_key,
                None,
                &lookup_value,
                None,
            )?;
        }

        let mut index = self.load_index(tenant)?;
        if !index.task_ids.iter().any(|value| value == &record.task_id) {
            index.task_ids.push(record.task_id.clone());
            index.task_ids.sort();
            self.save_index(tenant, &index)?;
        }

        Ok(())
    }

    fn list(&self, tenant: &TenantCtx) -> GResult<Vec<TaskRecord>> {
        let index = self.load_index(tenant)?;
        let mut records = Vec::with_capacity(index.task_ids.len());
        for task_id in index.task_ids {
            if let Some(record) = self.load(tenant, &task_id)? {
                records.push(record);
            }
        }
        Ok(records)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{StoreBackedTaskStore, TaskRecord, TaskStore, TaskStoreConfig};
    use greentic_state::inmemory::InMemoryStateStore;
    use greentic_types::{EnvId, TenantCtx, TenantId};
    use serde_json::json;

    fn tenant(value: &str) -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from(value).expect("tenant id"),
        )
    }

    #[test]
    fn task_store_roundtrips_records() {
        let store =
            StoreBackedTaskStore::new(InMemoryStateStore::new(), TaskStoreConfig::default());
        let tenant = tenant("tenant-a");
        let record = TaskRecord::new("task-1", json!({"status": "running"})).expect("record");

        store.save(&tenant, record).expect("save");
        let loaded = store.load(&tenant, "task-1").expect("load");
        assert_eq!(loaded.expect("record").state, json!({"status": "running"}));
    }

    #[test]
    fn task_store_list_is_tenant_scoped() {
        let store =
            StoreBackedTaskStore::new(InMemoryStateStore::new(), TaskStoreConfig::default());
        let tenant_a = tenant("tenant-a");
        let tenant_b = tenant("tenant-b");

        store
            .save(
                &tenant_a,
                TaskRecord::new("task-a", json!({"status": "a"})).expect("record"),
            )
            .expect("save");
        store
            .save(
                &tenant_b,
                TaskRecord::new("task-b", json!({"status": "b"})).expect("record"),
            )
            .expect("save");

        let list_a = store.list(&tenant_a).expect("list");
        let list_b = store.list(&tenant_b).expect("list");
        assert_eq!(list_a.len(), 1);
        assert_eq!(list_b.len(), 1);
        assert_eq!(list_a[0].task_id, "task-a");
        assert_eq!(list_b[0].task_id, "task-b");
    }
}
