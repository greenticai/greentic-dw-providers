use semver::Version;
use thiserror::Error;

use crate::{
    CapabilityDeclaration, CapabilityIdError, ProviderCategory, ProviderDeclSpec,
    capability_declaration, capability_id, capability_provider_ref, pack_capabilities_extension,
    provider_decl, provider_extension_inline,
};

use greentic_types::{
    CapabilitiesExtensionError, CborError, PackId, PackKind, PackManifest, PackSignatures,
    ProviderExtensionInline, encode_pack_manifest,
};

/// Short-term memory provider variants planned by this repository.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShortTermMemoryVariant {
    /// In-memory implementation.
    InMemory,
    /// Redis-backed implementation.
    Redis,
}

impl ShortTermMemoryVariant {
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
        format!("component:memory.{}", self.as_str())
    }

    /// Returns the provider type used in the provider manifest.
    #[must_use]
    pub fn provider_type(self) -> String {
        crate::provider_type(
            ProviderCategory::Memory,
            format!("short-term.{}", self.as_str()),
        )
    }
}

/// Errors produced while building short-term memory fixtures.
#[derive(Debug, Error)]
pub enum MemoryFixtureError {
    /// Capability identifier parsing failed.
    #[error(transparent)]
    CapabilityId(#[from] CapabilityIdError),
    /// Pack capability extension construction failed.
    #[error(transparent)]
    CapabilitiesExtension(#[from] CapabilitiesExtensionError),
    /// Pack CBOR encoding failed.
    #[error(transparent)]
    Cbor(#[from] CborError),
}

/// Returns the short-term memory capability URI.
#[must_use]
pub fn short_term_memory_capability_uri() -> String {
    crate::capability_uri(ProviderCategory::Memory, "short-term")
}

/// Returns the short-term memory capability identifier.
pub fn short_term_memory_capability_id()
-> Result<greentic_cap_types::CapabilityId, CapabilityIdError> {
    capability_id(ProviderCategory::Memory, "short-term")
}

/// Returns the short-term memory pack capability identifier.
#[must_use]
pub fn short_term_memory_pack_capability_id() -> String {
    crate::pack_capability_id(ProviderCategory::Memory, "short-term")
}

/// Returns the operations expected from a short-term memory provider.
#[must_use]
pub const fn short_term_memory_operations() -> [&'static str; 4] {
    ["memory.get", "memory.put", "memory.delete", "memory.clear"]
}

/// Returns the canonical provider declaration for a short-term memory backend.
#[must_use]
pub fn short_term_memory_provider_decl(
    variant: ShortTermMemoryVariant,
) -> greentic_types::ProviderDecl {
    provider_decl(ProviderDeclSpec {
        category: ProviderCategory::Memory,
        provider_name: format!("short-term.{}", variant.as_str()),
        capabilities: vec![short_term_memory_pack_capability_id()],
        ops: short_term_memory_operations()
            .into_iter()
            .map(str::to_string)
            .collect(),
        config_schema_ref: format!("schemas/memory/short-term/{}.json", variant.as_str()),
        state_schema_ref: Some(format!(
            "schemas/memory/short-term/{}-state.json",
            variant.as_str()
        )),
        component_ref: variant.component_ref(),
        export: "greentic_provider".to_string(),
        world: "greentic:provider/runtime".to_string(),
        docs_ref: Some(format!(
            "docs/providers/memory/short-term/{}.md",
            variant.as_str()
        )),
    })
}

/// Returns the canonical provider extension payload for a short-term memory backend.
#[must_use]
pub fn short_term_memory_provider_extension_inline(
    variants: Vec<ShortTermMemoryVariant>,
) -> ProviderExtensionInline {
    let providers = variants
        .into_iter()
        .map(short_term_memory_provider_decl)
        .collect();
    provider_extension_inline(providers)
}

/// Returns the short-term memory capability declaration for a provider variant.
pub fn short_term_memory_capability_declaration(
    variant: ShortTermMemoryVariant,
) -> Result<CapabilityDeclaration, MemoryFixtureError> {
    let capability = short_term_memory_capability_id()?;
    let mut offer = greentic_cap_types::CapabilityOffer::new(
        format!("offer.short-term.{}", variant.as_str()),
        capability,
    );
    offer.provider = Some(capability_provider_ref(
        variant.component_ref(),
        "memory.get",
    ));
    Ok(capability_declaration(
        vec![offer],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
}

/// Returns the pack capability extension payload for a short-term memory backend.
#[must_use]
pub fn short_term_memory_pack_capabilities(
    variant: ShortTermMemoryVariant,
) -> greentic_types::CapabilitiesExtensionV1 {
    pack_capabilities_extension(
        ProviderCategory::Memory,
        "short-term",
        format!("offer.short-term.{}", variant.as_str()),
        variant.component_ref(),
        "memory.get",
    )
}

/// Builds a pack manifest for a short-term memory backend.
pub fn short_term_memory_pack_manifest(
    pack_id: PackId,
    variant: ShortTermMemoryVariant,
) -> Result<PackManifest, MemoryFixtureError> {
    let provider_decl = short_term_memory_provider_decl(variant);
    let mut manifest = PackManifest {
        schema_version: "pack-v1".to_string(),
        pack_id,
        name: Some(format!("memory short-term {}", variant.as_str())),
        version: Version::new(0, 4, 0),
        kind: PackKind::Provider,
        publisher: "greentic".to_string(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: vec![greentic_types::ComponentCapability {
            name: short_term_memory_pack_capability_id(),
            description: Some("short-term memory capability".to_string()),
        }],
        secret_requirements: Vec::new(),
        signatures: PackSignatures::default(),
        bootstrap: None,
        extensions: None,
    };

    manifest
        .ensure_provider_extension_inline()
        .providers
        .push(provider_decl);
    manifest
        .set_capabilities_extension_v1(short_term_memory_pack_capabilities(variant))
        .map_err(MemoryFixtureError::from)?;
    Ok(manifest)
}

/// Encodes a short-term memory pack manifest to CBOR bytes.
pub fn short_term_memory_pack_manifest_cbor(
    pack_id: PackId,
    variant: ShortTermMemoryVariant,
) -> Result<Vec<u8>, MemoryFixtureError> {
    let manifest = short_term_memory_pack_manifest(pack_id, variant)?;
    Ok(encode_pack_manifest(&manifest)?)
}

// NOTE: ProviderCatalog wiring intentionally deferred until unified-catalog lands.

/// Long-term memory provider variants planned by this repository.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LongTermMemoryVariant {
    /// Chronicle graph-based implementation.
    Chronicle,
}

impl LongTermMemoryVariant {
    /// Returns the variant as a lowercase string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Chronicle => "chronicle",
        }
    }

    /// Returns the provider component reference used by this variant.
    #[must_use]
    pub fn component_ref(self) -> String {
        format!("component:memory.{}", self.as_str())
    }

    /// Returns the provider type used in the provider manifest.
    #[must_use]
    pub fn provider_type(self) -> String {
        crate::provider_type(
            ProviderCategory::Memory,
            format!("long-term.{}", self.as_str()),
        )
    }
}

/// Returns the long-term memory capability URI.
#[must_use]
pub fn long_term_memory_capability_uri() -> String {
    crate::capability_uri(ProviderCategory::Memory, "long-term")
}

/// Returns the long-term memory capability identifier.
pub fn long_term_memory_capability_id()
-> Result<greentic_cap_types::CapabilityId, CapabilityIdError> {
    capability_id(ProviderCategory::Memory, "long-term")
}

/// Returns the long-term memory pack capability identifier.
#[must_use]
pub fn long_term_memory_pack_capability_id() -> String {
    crate::pack_capability_id(ProviderCategory::Memory, "long-term")
}

/// Operations match the LongTermMemory trait surface (ingest_episode -> memory.ingest, recall -> memory.recall). Extend only when the trait grows.
#[must_use]
pub const fn long_term_memory_operations() -> [&'static str; 2] {
    ["memory.ingest", "memory.recall"]
}

/// Returns the canonical provider declaration for a long-term memory backend.
#[must_use]
pub fn long_term_memory_provider_decl(
    variant: LongTermMemoryVariant,
) -> greentic_types::ProviderDecl {
    provider_decl(ProviderDeclSpec {
        category: ProviderCategory::Memory,
        provider_name: format!("long-term.{}", variant.as_str()),
        capabilities: vec![long_term_memory_pack_capability_id()],
        ops: long_term_memory_operations()
            .into_iter()
            .map(str::to_string)
            .collect(),
        config_schema_ref: format!("schemas/memory/long-term/{}.json", variant.as_str()),
        state_schema_ref: Some(format!(
            "schemas/memory/long-term/{}-state.json",
            variant.as_str()
        )),
        component_ref: variant.component_ref(),
        export: "greentic_provider".to_string(),
        world: "greentic:provider/runtime".to_string(),
        docs_ref: Some(format!(
            "docs/providers/memory/long-term/{}.md",
            variant.as_str()
        )),
    })
}

/// Returns the canonical provider extension payload for a long-term memory backend.
#[must_use]
pub fn long_term_memory_provider_extension_inline(
    variants: Vec<LongTermMemoryVariant>,
) -> ProviderExtensionInline {
    let providers = variants
        .into_iter()
        .map(long_term_memory_provider_decl)
        .collect();
    provider_extension_inline(providers)
}

/// Returns the long-term memory capability declaration for a provider variant.
pub fn long_term_memory_capability_declaration(
    variant: LongTermMemoryVariant,
) -> Result<CapabilityDeclaration, MemoryFixtureError> {
    let capability = long_term_memory_capability_id()?;
    let mut offer = greentic_cap_types::CapabilityOffer::new(
        format!("offer.long-term.{}", variant.as_str()),
        capability,
    );
    offer.provider = Some(capability_provider_ref(
        variant.component_ref(),
        "memory.ingest",
    ));
    Ok(capability_declaration(
        vec![offer],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
}

/// Returns the pack capability extension payload for a long-term memory backend.
#[must_use]
pub fn long_term_memory_pack_capabilities(
    variant: LongTermMemoryVariant,
) -> greentic_types::CapabilitiesExtensionV1 {
    pack_capabilities_extension(
        ProviderCategory::Memory,
        "long-term",
        format!("offer.long-term.{}", variant.as_str()),
        variant.component_ref(),
        "memory.ingest",
    )
}

/// Builds a pack manifest for a long-term memory backend.
pub fn long_term_memory_pack_manifest(
    pack_id: PackId,
    variant: LongTermMemoryVariant,
) -> Result<PackManifest, MemoryFixtureError> {
    let provider_decl = long_term_memory_provider_decl(variant);
    let mut manifest = PackManifest {
        schema_version: "pack-v1".to_string(),
        pack_id,
        name: Some(format!("memory long-term {}", variant.as_str())),
        version: Version::new(0, 4, 0),
        kind: PackKind::Provider,
        publisher: "greentic".to_string(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: vec![greentic_types::ComponentCapability {
            name: long_term_memory_pack_capability_id(),
            description: Some("long-term memory capability".to_string()),
        }],
        secret_requirements: Vec::new(),
        signatures: PackSignatures::default(),
        bootstrap: None,
        extensions: None,
    };

    manifest
        .ensure_provider_extension_inline()
        .providers
        .push(provider_decl);
    manifest
        .set_capabilities_extension_v1(long_term_memory_pack_capabilities(variant))
        .map_err(MemoryFixtureError::from)?;
    Ok(manifest)
}

/// Encodes a long-term memory pack manifest to CBOR bytes.
pub fn long_term_memory_pack_manifest_cbor(
    pack_id: PackId,
    variant: LongTermMemoryVariant,
) -> Result<Vec<u8>, MemoryFixtureError> {
    let manifest = long_term_memory_pack_manifest(pack_id, variant)?;
    Ok(encode_pack_manifest(&manifest)?)
}
