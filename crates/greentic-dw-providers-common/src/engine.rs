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

/// Engine provider variants planned by this repository.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EngineVariant {
    /// Simple default decision engine.
    Default,
    /// Router/planner-lite engine.
    RouterLite,
}

impl EngineVariant {
    /// Returns the variant as a lowercase string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::RouterLite => "router-lite",
        }
    }

    /// Returns the provider component reference used by this variant.
    #[must_use]
    pub fn component_ref(self) -> String {
        format!("component:engine.{}", self.as_str())
    }

    /// Returns the provider type used in the provider manifest.
    #[must_use]
    pub fn provider_type(self) -> String {
        crate::provider_type(ProviderCategory::Engine, self.as_str())
    }
}

/// Errors produced while building engine fixtures.
#[derive(Debug, Error)]
pub enum EngineFixtureError {
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

/// Returns the engine capability URI for the provided engine contract.
#[must_use]
pub fn engine_capability_uri(capability: impl AsRef<str>) -> String {
    crate::capability_uri(ProviderCategory::Engine, capability)
}

/// Returns the canonical engine capability identifier.
pub fn engine_capability_id(
    capability: impl AsRef<str>,
) -> Result<greentic_cap_types::CapabilityId, CapabilityIdError> {
    capability_id(ProviderCategory::Engine, capability)
}

/// Returns the canonical engine pack capability identifier.
#[must_use]
pub fn engine_pack_capability_id(capability: impl AsRef<str>) -> String {
    crate::pack_capability_id(ProviderCategory::Engine, capability)
}

/// Resolves an engine variant from a pack capability id.
#[must_use]
pub fn engine_variant_from_pack_capability_id(
    capability_id: impl AsRef<str>,
) -> Option<EngineVariant> {
    match capability_id.as_ref() {
        "greentic.cap.engine.default" => Some(EngineVariant::Default),
        "greentic.cap.engine.router" => Some(EngineVariant::RouterLite),
        _ => None,
    }
}

/// Returns the operations expected from an engine provider.
#[must_use]
pub const fn engine_operations() -> [&'static str; 2] {
    ["engine.decide", "engine.route"]
}

fn engine_operation_for_variant(variant: EngineVariant) -> &'static str {
    match variant {
        EngineVariant::Default => "engine.decide",
        EngineVariant::RouterLite => "engine.route",
    }
}

/// Returns the canonical provider declaration for an engine backend.
#[must_use]
pub fn engine_provider_decl(variant: EngineVariant) -> greentic_types::ProviderDecl {
    provider_decl(ProviderDeclSpec {
        category: ProviderCategory::Engine,
        provider_name: variant.as_str().to_string(),
        capabilities: vec![engine_pack_capability_id(match variant {
            EngineVariant::Default => "default",
            EngineVariant::RouterLite => "router",
        })],
        ops: match variant {
            EngineVariant::Default => vec!["engine.decide".to_string()],
            EngineVariant::RouterLite => vec![
                "engine.decide".to_string(),
                "engine.route".to_string(),
                "engine.plan".to_string(),
            ],
        },
        config_schema_ref: format!("schemas/engine/{}.json", variant.as_str()),
        state_schema_ref: None,
        component_ref: variant.component_ref(),
        export: "greentic_provider".to_string(),
        world: "greentic:provider/runtime".to_string(),
        docs_ref: Some(format!("docs/providers/engine/{}.md", variant.as_str())),
    })
}

/// Returns the canonical provider extension payload for an engine backend.
#[must_use]
pub fn engine_provider_extension_inline(variants: Vec<EngineVariant>) -> ProviderExtensionInline {
    let providers = variants.into_iter().map(engine_provider_decl).collect();
    provider_extension_inline(providers)
}

/// Returns the engine capability declaration for a provider variant.
pub fn engine_capability_declaration(
    variant: EngineVariant,
) -> Result<CapabilityDeclaration, EngineFixtureError> {
    let capability = engine_capability_id(match variant {
        EngineVariant::Default => "default",
        EngineVariant::RouterLite => "router",
    })?;
    let mut offer = greentic_cap_types::CapabilityOffer::new(
        format!("offer.engine.{}", variant.as_str()),
        capability,
    );
    offer.provider = Some(capability_provider_ref(
        variant.component_ref(),
        engine_operation_for_variant(variant),
    ));
    Ok(capability_declaration(
        vec![offer],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
}

/// Returns the pack capability payload for an engine backend.
#[must_use]
pub fn engine_pack_capabilities(variant: EngineVariant) -> greentic_types::CapabilitiesExtensionV1 {
    pack_capabilities_extension(
        ProviderCategory::Engine,
        match variant {
            EngineVariant::Default => "default",
            EngineVariant::RouterLite => "router",
        },
        format!("offer.engine.{}", variant.as_str()),
        variant.component_ref(),
        engine_operation_for_variant(variant),
    )
}

/// Builds a pack manifest for an engine backend.
pub fn engine_pack_manifest(
    pack_id: PackId,
    variant: EngineVariant,
) -> Result<PackManifest, EngineFixtureError> {
    let provider_decl = engine_provider_decl(variant);
    let mut manifest = PackManifest {
        schema_version: "pack-v1".to_string(),
        pack_id,
        name: Some(format!("engine {}", variant.as_str())),
        version: Version::new(0, 4, 0),
        kind: PackKind::Provider,
        publisher: "greentic".to_string(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: vec![greentic_types::ComponentCapability {
            name: engine_pack_capability_id(match variant {
                EngineVariant::Default => "default",
                EngineVariant::RouterLite => "router",
            }),
            description: Some("engine capability".to_string()),
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
        .set_capabilities_extension_v1(engine_pack_capabilities(variant))
        .map_err(EngineFixtureError::from)?;
    Ok(manifest)
}

/// Encodes an engine pack manifest to CBOR bytes.
pub fn engine_pack_manifest_cbor(
    pack_id: PackId,
    variant: EngineVariant,
) -> Result<Vec<u8>, EngineFixtureError> {
    let manifest = engine_pack_manifest(pack_id, variant)?;
    Ok(encode_pack_manifest(&manifest)?)
}
