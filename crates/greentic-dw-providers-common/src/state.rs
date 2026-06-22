use semver::Version;
use thiserror::Error;

use crate::{
    ProviderCategory, ProviderDeclSpec, capability_id, capability_uri, pack_capabilities_extension,
    provider_decl, provider_extension_inline,
};

use greentic_types::{
    CapabilitiesExtensionError, CborError, PackId, PackKind, PackManifest, PackSignatures,
    StateBackendKind, StateKey, StatePath, encode_pack_manifest,
};

/// Task-state provider variants planned by this repository.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskStoreVariant {
    /// In-memory task store.
    InMemory,
    /// Redis-backed task store.
    Redis,
}

impl TaskStoreVariant {
    /// Returns the variant as a lowercase string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InMemory => "in-memory",
            Self::Redis => "redis",
        }
    }

    /// Returns the provider component reference used by this variant.
    #[must_use]
    pub fn component_ref(self) -> String {
        format!("component:state.task-store.{}", self.as_str())
    }

    /// Returns the provider type used in the provider manifest.
    #[must_use]
    pub fn provider_type(self) -> String {
        crate::provider_type(
            ProviderCategory::State,
            format!("task-store.{}", self.as_str()),
        )
    }
}

/// Errors produced while building task-state fixtures.
#[derive(Debug, Error)]
pub enum StateFixtureError {
    /// Capability identifier parsing failed.
    #[error(transparent)]
    CapabilityId(#[from] crate::CapabilityIdError),
    /// Pack capability extension construction failed.
    #[error(transparent)]
    CapabilitiesExtension(#[from] CapabilitiesExtensionError),
    /// Pack CBOR encoding failed.
    #[error(transparent)]
    Cbor(#[from] CborError),
}

/// Returns the task-state capability URI.
#[must_use]
pub fn task_store_capability_uri() -> String {
    capability_uri(ProviderCategory::State, "task-store")
}

/// Returns the task-state capability identifier.
pub fn task_store_capability_id()
-> Result<greentic_cap_types::CapabilityId, crate::CapabilityIdError> {
    capability_id(ProviderCategory::State, "task-store")
}

/// Returns the task-state pack capability identifier.
#[must_use]
pub fn task_store_pack_capability_id() -> String {
    crate::pack_capability_id(ProviderCategory::State, "task-store")
}

/// Returns the operations expected from a task-state provider.
#[must_use]
pub const fn task_store_operations() -> [&'static str; 3] {
    ["state.load", "state.save", "state.list"]
}

/// Returns the canonical provider declaration for a task-state backend.
#[must_use]
pub fn task_store_provider_decl(variant: TaskStoreVariant) -> greentic_types::ProviderDecl {
    provider_decl(ProviderDeclSpec {
        category: ProviderCategory::State,
        provider_name: format!("task-store.{}", variant.as_str()),
        capabilities: vec![task_store_pack_capability_id()],
        ops: task_store_operations()
            .into_iter()
            .map(str::to_string)
            .collect(),
        config_schema_ref: format!("schemas/state/task-store/{}.json", variant.as_str()),
        state_schema_ref: Some(format!(
            "schemas/state/task-store/{}-state.json",
            variant.as_str()
        )),
        component_ref: variant.component_ref(),
        export: "greentic_provider".to_string(),
        world: "greentic:provider/runtime".to_string(),
        docs_ref: Some(format!(
            "docs/providers/state/task-store/{}.md",
            variant.as_str()
        )),
    })
}

/// Returns the canonical provider extension payload for task-state backends.
#[must_use]
pub fn task_store_provider_extension_inline(
    variants: Vec<TaskStoreVariant>,
) -> greentic_types::ProviderExtensionInline {
    let providers = variants.into_iter().map(task_store_provider_decl).collect();
    provider_extension_inline(providers)
}

/// Returns the shared state backend kind for a task-state backend.
#[must_use]
pub fn task_store_backend_kind(
    variant: TaskStoreVariant,
    redis_url: Option<String>,
) -> StateBackendKind {
    match variant {
        TaskStoreVariant::InMemory => StateBackendKind::Memory {
            max_entries: 0,
            default_ttl_seconds: 0,
        },
        TaskStoreVariant::Redis => StateBackendKind::Redis {
            redis_url: redis_url.unwrap_or_else(|| "redis://localhost:6379/0".to_string()),
            key_prefix: "greentic".to_string(),
            default_ttl_seconds: 0,
            pool_size: 5,
            tls_enabled: false,
        },
    }
}

fn state_path(mut segments: Vec<String>) -> StatePath {
    StatePath {
        segments: {
            let mut path = StatePath::root();
            path.segments.append(&mut segments);
            path.segments
        },
    }
}

/// Returns the checkpoint key used for a task id.
#[must_use]
pub fn task_store_checkpoint_key(task_id: impl AsRef<str>) -> StateKey {
    StateKey::new(format!("dw:state:checkpoint:{}", task_id.as_ref()))
}

/// Returns the resume lookup key used for a task id.
#[must_use]
pub fn task_store_resume_lookup_key(task_id: impl AsRef<str>) -> StateKey {
    StateKey::new(format!("dw:state:resume:{}", task_id.as_ref()))
}

/// Returns the path used to store checkpoint metadata.
#[must_use]
pub fn task_store_checkpoint_path(task_id: impl AsRef<str>) -> StatePath {
    state_path(vec![
        "state".to_string(),
        "task-store".to_string(),
        "checkpoint".to_string(),
        task_id.as_ref().to_string(),
    ])
}

/// Returns the path used to store pending-await metadata.
#[must_use]
pub fn task_store_pending_await_path(task_id: impl AsRef<str>) -> StatePath {
    state_path(vec![
        "state".to_string(),
        "task-store".to_string(),
        "pending-await".to_string(),
        task_id.as_ref().to_string(),
    ])
}

/// Returns the pack capability payload for a task-state backend.
#[must_use]
pub fn task_store_pack_capabilities(
    variant: TaskStoreVariant,
) -> greentic_types::CapabilitiesExtensionV1 {
    pack_capabilities_extension(
        ProviderCategory::State,
        "task-store",
        format!("offer.task-store.{}", variant.as_str()),
        variant.component_ref(),
        "state.save",
    )
}

/// Builds a pack manifest for a task-state backend.
pub fn task_store_pack_manifest(
    pack_id: PackId,
    variant: TaskStoreVariant,
) -> Result<PackManifest, StateFixtureError> {
    let provider_decl = task_store_provider_decl(variant);
    let mut manifest = PackManifest {
        schema_version: "pack-v1".to_string(),
        pack_id,
        name: Some(format!("state task-store {}", variant.as_str())),
        version: Version::new(0, 4, 0),
        kind: PackKind::Provider,
        publisher: "greentic".to_string(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: vec![greentic_types::ComponentCapability {
            name: task_store_pack_capability_id(),
            description: Some("task-state capability".to_string()),
        }],
        secret_requirements: Vec::new(),
        signatures: PackSignatures::default(),
        bootstrap: None,
        extensions: None,
        agents: Default::default(),
    };

    manifest
        .ensure_provider_extension_inline()
        .providers
        .push(provider_decl);
    manifest
        .set_capabilities_extension_v1(task_store_pack_capabilities(variant))
        .map_err(StateFixtureError::from)?;
    Ok(manifest)
}

/// Encodes a task-state pack manifest to CBOR bytes.
pub fn task_store_pack_manifest_cbor(
    pack_id: PackId,
    variant: TaskStoreVariant,
) -> Result<Vec<u8>, StateFixtureError> {
    let manifest = task_store_pack_manifest(pack_id, variant)?;
    Ok(encode_pack_manifest(&manifest)?)
}
