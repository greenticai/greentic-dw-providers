#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Shared short-term memory contract and generic store-backed implementation.

use greentic_state::StateStore;
use greentic_types::{ErrorCode, GResult, GreenticError, StateKey, TenantCtx};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Default prefix used to scope short-term memory entries in the shared state store.
pub const DEFAULT_SHORT_TERM_MEMORY_PREFIX: &str = "dw.memory.short-term";

/// Configuration shared by short-term memory backends.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortTermMemoryConfig {
    /// Store prefix used to isolate this memory family.
    pub prefix: String,
    /// Default TTL applied when a write does not specify one.
    pub default_ttl_seconds: Option<u32>,
}

impl Default for ShortTermMemoryConfig {
    fn default() -> Self {
        Self {
            prefix: DEFAULT_SHORT_TERM_MEMORY_PREFIX.to_string(),
            default_ttl_seconds: None,
        }
    }
}

impl ShortTermMemoryConfig {
    /// Returns a copy with a custom store prefix.
    #[must_use]
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Returns a copy with a custom default TTL.
    #[must_use]
    pub fn with_default_ttl(mut self, ttl_seconds: Option<u32>) -> Self {
        self.default_ttl_seconds = ttl_seconds;
        self
    }
}

/// Write request for a short-term memory entry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShortTermMemoryPutRequest {
    /// Caller-provided key within the memory scope.
    pub key: String,
    /// JSON payload to store.
    pub value: Value,
    /// Optional TTL override for this write.
    pub ttl_seconds: Option<u32>,
}

impl ShortTermMemoryPutRequest {
    /// Creates a new write request.
    #[must_use]
    pub fn new(key: impl Into<String>, value: Value) -> Self {
        Self {
            key: key.into(),
            value,
            ttl_seconds: None,
        }
    }

    /// Sets an explicit TTL for this write.
    #[must_use]
    pub fn with_ttl(mut self, ttl_seconds: Option<u32>) -> Self {
        self.ttl_seconds = ttl_seconds;
        self
    }
}

/// Contract implemented by short-term memory providers.
pub trait ShortTermMemory: Send + Sync {
    /// Fetches the JSON value for a short-term memory key.
    fn get(&self, tenant: &TenantCtx, key: &str) -> GResult<Option<Value>>;

    /// Upserts a short-term memory value.
    fn put(&self, tenant: &TenantCtx, request: ShortTermMemoryPutRequest) -> GResult<()>;

    /// Deletes a single short-term memory entry.
    fn delete(&self, tenant: &TenantCtx, key: &str) -> GResult<bool>;

    /// Clears all short-term memory entries for the given tenant and provider prefix.
    fn clear(&self, tenant: &TenantCtx) -> GResult<u64>;
}

/// Generic short-term memory implementation backed by a [`StateStore`].
pub struct StoreBackedShortTermMemory<S> {
    store: S,
    config: ShortTermMemoryConfig,
}

impl<S> StoreBackedShortTermMemory<S> {
    /// Creates a new store-backed short-term memory facade.
    #[must_use]
    pub fn new(store: S, config: ShortTermMemoryConfig) -> Self {
        Self { store, config }
    }

    /// Returns the active configuration.
    #[must_use]
    pub fn config(&self) -> &ShortTermMemoryConfig {
        &self.config
    }

    /// Returns the inner store.
    #[must_use]
    pub fn store(&self) -> &S {
        &self.store
    }
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}

fn state_key_for(memory_key: &str) -> GResult<StateKey> {
    let trimmed = memory_key.trim();
    if trimmed.is_empty() {
        return Err(invalid("short-term memory key must not be empty"));
    }

    Ok(StateKey::new(format!("memory:short-term:{trimmed}")))
}

impl<S> ShortTermMemory for StoreBackedShortTermMemory<S>
where
    S: StateStore,
{
    fn get(&self, tenant: &TenantCtx, key: &str) -> GResult<Option<Value>> {
        let state_key = state_key_for(key)?;
        self.store
            .get_json(tenant, &self.config.prefix, &state_key, None)
    }

    fn put(&self, tenant: &TenantCtx, request: ShortTermMemoryPutRequest) -> GResult<()> {
        let state_key = state_key_for(&request.key)?;
        let ttl = request.ttl_seconds.or(self.config.default_ttl_seconds);
        self.store.set_json(
            tenant,
            &self.config.prefix,
            &state_key,
            None,
            &request.value,
            ttl,
        )
    }

    fn delete(&self, tenant: &TenantCtx, key: &str) -> GResult<bool> {
        let state_key = state_key_for(key)?;
        self.store.del(tenant, &self.config.prefix, &state_key)
    }

    fn clear(&self, tenant: &TenantCtx) -> GResult<u64> {
        self.store.del_prefix(tenant, &self.config.prefix)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        ShortTermMemory, ShortTermMemoryConfig, ShortTermMemoryPutRequest,
        StoreBackedShortTermMemory,
    };
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
    fn store_backed_memory_roundtrips_values() {
        let memory = StoreBackedShortTermMemory::new(
            InMemoryStateStore::new(),
            ShortTermMemoryConfig::default(),
        );
        let tenant = tenant("tenant-a");

        memory
            .put(
                &tenant,
                ShortTermMemoryPutRequest::new("latest-user-message", json!({"text": "hello"})),
            )
            .expect("put succeeds");

        let value = memory
            .get(&tenant, "latest-user-message")
            .expect("get succeeds");
        assert_eq!(value, Some(json!({"text": "hello"})));
    }

    #[test]
    fn clear_is_tenant_scoped() {
        let memory = StoreBackedShortTermMemory::new(
            InMemoryStateStore::new(),
            ShortTermMemoryConfig::default(),
        );
        let tenant_a = tenant("tenant-a");
        let tenant_b = tenant("tenant-b");

        memory
            .put(
                &tenant_a,
                ShortTermMemoryPutRequest::new("note", json!("a")),
            )
            .expect("put tenant a");
        memory
            .put(
                &tenant_b,
                ShortTermMemoryPutRequest::new("note", json!("b")),
            )
            .expect("put tenant b");

        let deleted = memory.clear(&tenant_a).expect("clear succeeds");
        assert_eq!(deleted, 1);
        assert_eq!(memory.get(&tenant_a, "note").expect("get"), None);
        assert_eq!(
            memory.get(&tenant_b, "note").expect("get"),
            Some(json!("b"))
        );
    }

    #[test]
    fn delete_removes_single_entry() {
        let memory = StoreBackedShortTermMemory::new(
            InMemoryStateStore::new(),
            ShortTermMemoryConfig::default(),
        );
        let tenant = tenant("tenant-a");

        memory
            .put(
                &tenant,
                ShortTermMemoryPutRequest::new("note", json!("value")),
            )
            .expect("put succeeds");

        assert!(memory.delete(&tenant, "note").expect("delete succeeds"));
        assert_eq!(memory.get(&tenant, "note").expect("get succeeds"), None);
    }
}
