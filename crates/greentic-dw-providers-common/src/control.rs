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

/// Control provider variants planned by this repository.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlVariant {
    /// Basic policy provider.
    BasicPolicy,
    /// Delegation guard provider.
    DelegationGuard,
}

impl ControlVariant {
    /// Returns the variant as a lowercase string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BasicPolicy => "basic-policy",
            Self::DelegationGuard => "delegation-guard",
        }
    }

    /// Returns the canonical capability name for the variant.
    #[must_use]
    pub const fn capability_name(self) -> &'static str {
        match self {
            Self::BasicPolicy => "basic",
            Self::DelegationGuard => "delegation-guard",
        }
    }

    /// Returns the provider component reference used by this variant.
    #[must_use]
    pub fn component_ref(self) -> String {
        format!("component:control.{}", self.as_str())
    }

    /// Returns the provider type used in the provider manifest.
    #[must_use]
    pub fn provider_type(self) -> String {
        crate::provider_type(ProviderCategory::Control, self.as_str())
    }
}

/// Errors produced while building control fixtures.
#[derive(Debug, Error)]
pub enum ControlFixtureError {
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

/// Returns the canonical control capability URI for a local capability name.
#[must_use]
pub fn control_capability_uri(capability: impl AsRef<str>) -> String {
    crate::capability_uri(ProviderCategory::Control, capability)
}

/// Returns the canonical control capability identifier.
pub fn control_capability_id(
    capability: impl AsRef<str>,
) -> Result<greentic_cap_types::CapabilityId, CapabilityIdError> {
    capability_id(ProviderCategory::Control, capability)
}

/// Returns the canonical control pack capability identifier.
#[must_use]
pub fn control_pack_capability_id(capability: impl AsRef<str>) -> String {
    crate::pack_capability_id(ProviderCategory::Control, capability)
}

/// Resolves a control variant from a pack capability id.
#[must_use]
pub fn control_variant_from_pack_capability_id(
    capability_id: impl AsRef<str>,
) -> Option<ControlVariant> {
    match capability_id.as_ref() {
        "greentic.cap.control.basic" => Some(ControlVariant::BasicPolicy),
        "greentic.cap.control.delegation-guard" => Some(ControlVariant::DelegationGuard),
        _ => None,
    }
}

/// Returns the operations expected from a control provider.
#[must_use]
pub const fn control_operations() -> [&'static str; 2] {
    ["control.evaluate", "control.guard"]
}

fn control_operation_for_variant(variant: ControlVariant) -> &'static str {
    match variant {
        ControlVariant::BasicPolicy => "control.evaluate",
        ControlVariant::DelegationGuard => "control.guard",
    }
}

/// Returns the canonical provider declaration for a control backend.
#[must_use]
pub fn control_provider_decl(variant: ControlVariant) -> greentic_types::ProviderDecl {
    provider_decl(ProviderDeclSpec {
        category: ProviderCategory::Control,
        provider_name: variant.as_str().to_string(),
        capabilities: vec![control_pack_capability_id(variant.capability_name())],
        ops: control_operations()
            .into_iter()
            .map(str::to_string)
            .collect(),
        config_schema_ref: format!("schemas/control/{}.json", variant.as_str()),
        state_schema_ref: None,
        component_ref: variant.component_ref(),
        export: "greentic_provider".to_string(),
        world: "greentic:provider/runtime".to_string(),
        docs_ref: Some(format!("docs/providers/control/{}.md", variant.as_str())),
    })
}

/// Returns the canonical provider extension payload for a control backend.
#[must_use]
pub fn control_provider_extension_inline(variants: Vec<ControlVariant>) -> ProviderExtensionInline {
    let providers = variants.into_iter().map(control_provider_decl).collect();
    provider_extension_inline(providers)
}

/// Returns the control capability declaration for a provider variant.
pub fn control_capability_declaration(
    variant: ControlVariant,
) -> Result<CapabilityDeclaration, ControlFixtureError> {
    let capability = control_capability_id(variant.capability_name())?;
    let mut offer = greentic_cap_types::CapabilityOffer::new(
        format!("offer.control.{}", variant.as_str()),
        capability,
    );
    offer.provider = Some(capability_provider_ref(
        variant.component_ref(),
        control_operation_for_variant(variant),
    ));
    Ok(capability_declaration(
        vec![offer],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
}

/// Returns the pack capability payload for a control backend.
#[must_use]
pub fn control_pack_capabilities(
    variant: ControlVariant,
) -> greentic_types::CapabilitiesExtensionV1 {
    pack_capabilities_extension(
        ProviderCategory::Control,
        variant.capability_name(),
        format!("offer.control.{}", variant.as_str()),
        variant.component_ref(),
        control_operation_for_variant(variant),
    )
}

/// Builds a pack manifest for a control backend.
pub fn control_pack_manifest(
    pack_id: PackId,
    variant: ControlVariant,
) -> Result<PackManifest, ControlFixtureError> {
    let provider_decl = control_provider_decl(variant);
    let mut manifest = PackManifest {
        // `PackManifest.agents` exists only in the greentic-types revision the
        // long-term-memory consumer (greentic-runner) pins; gate it behind the
        // `pack-manifest-agents` feature so this crate also builds against the
        // published greentic-types that lacks the field. No agent blobs here.
        #[cfg(feature = "pack-manifest-agents")]
        agents: Default::default(),
        schema_version: "pack-v1".to_string(),
        pack_id,
        name: Some(format!("control {}", variant.as_str())),
        version: Version::new(0, 4, 0),
        kind: PackKind::Provider,
        publisher: "greentic".to_string(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: vec![greentic_types::ComponentCapability {
            name: control_pack_capability_id(variant.capability_name()),
            description: Some("control capability".to_string()),
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
        .set_capabilities_extension_v1(control_pack_capabilities(variant))
        .map_err(ControlFixtureError::from)?;
    Ok(manifest)
}

/// Encodes a control pack manifest to CBOR bytes.
pub fn control_pack_manifest_cbor(
    pack_id: PackId,
    variant: ControlVariant,
) -> Result<Vec<u8>, ControlFixtureError> {
    let manifest = control_pack_manifest(pack_id, variant)?;
    Ok(encode_pack_manifest(&manifest)?)
}
