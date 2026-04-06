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

/// Observer provider variants planned by this repository.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObserverVariant {
    /// Basic audit observer.
    BasicAudit,
    /// Basic metrics observer.
    BasicMetrics,
}

impl ObserverVariant {
    /// Returns the variant as a lowercase string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BasicAudit => "basic-audit",
            Self::BasicMetrics => "basic-metrics",
        }
    }

    /// Returns the canonical capability name for the variant.
    #[must_use]
    pub const fn capability_name(self) -> &'static str {
        match self {
            Self::BasicAudit => "audit",
            Self::BasicMetrics => "metrics",
        }
    }

    /// Returns the provider component reference used by this variant.
    #[must_use]
    pub fn component_ref(self) -> String {
        format!("component:observer.{}", self.as_str())
    }

    /// Returns the provider type used in the provider manifest.
    #[must_use]
    pub fn provider_type(self) -> String {
        crate::provider_type(ProviderCategory::Observer, self.as_str())
    }
}

/// Errors produced while building observer fixtures.
#[derive(Debug, Error)]
pub enum ObserverFixtureError {
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

/// Returns the canonical observer capability URI for a local capability name.
#[must_use]
pub fn observer_capability_uri(capability: impl AsRef<str>) -> String {
    crate::capability_uri(ProviderCategory::Observer, capability)
}

/// Returns the canonical observer capability identifier.
pub fn observer_capability_id(
    capability: impl AsRef<str>,
) -> Result<greentic_cap_types::CapabilityId, CapabilityIdError> {
    capability_id(ProviderCategory::Observer, capability)
}

/// Returns the canonical observer pack capability identifier.
#[must_use]
pub fn observer_pack_capability_id(capability: impl AsRef<str>) -> String {
    crate::pack_capability_id(ProviderCategory::Observer, capability)
}

/// Resolves an observer variant from a pack capability id.
#[must_use]
pub fn observer_variant_from_pack_capability_id(
    capability_id: impl AsRef<str>,
) -> Option<ObserverVariant> {
    match capability_id.as_ref() {
        "greentic.cap.observer.audit" => Some(ObserverVariant::BasicAudit),
        "greentic.cap.observer.metrics" => Some(ObserverVariant::BasicMetrics),
        _ => None,
    }
}

/// Returns the operations expected from an observer provider.
#[must_use]
pub const fn observer_operations() -> [&'static str; 2] {
    ["observer.observe", "observer.report"]
}

fn observer_operation_for_variant(variant: ObserverVariant) -> &'static str {
    match variant {
        ObserverVariant::BasicAudit => "observer.observe",
        ObserverVariant::BasicMetrics => "observer.report",
    }
}

/// Returns the canonical provider declaration for an observer backend.
#[must_use]
pub fn observer_provider_decl(variant: ObserverVariant) -> greentic_types::ProviderDecl {
    provider_decl(ProviderDeclSpec {
        category: ProviderCategory::Observer,
        provider_name: variant.as_str().to_string(),
        capabilities: vec![observer_pack_capability_id(variant.capability_name())],
        ops: observer_operations()
            .into_iter()
            .map(str::to_string)
            .collect(),
        config_schema_ref: format!("schemas/observer/{}.json", variant.as_str()),
        state_schema_ref: None,
        component_ref: variant.component_ref(),
        export: "greentic_provider".to_string(),
        world: "greentic:provider/runtime".to_string(),
        docs_ref: Some(format!("docs/providers/observer/{}.md", variant.as_str())),
    })
}

/// Returns the canonical provider extension payload for an observer backend.
#[must_use]
pub fn observer_provider_extension_inline(
    variants: Vec<ObserverVariant>,
) -> ProviderExtensionInline {
    let providers = variants.into_iter().map(observer_provider_decl).collect();
    provider_extension_inline(providers)
}

/// Returns the observer capability declaration for a provider variant.
pub fn observer_capability_declaration(
    variant: ObserverVariant,
) -> Result<CapabilityDeclaration, ObserverFixtureError> {
    let capability = observer_capability_id(variant.capability_name())?;
    let mut offer = greentic_cap_types::CapabilityOffer::new(
        format!("offer.observer.{}", variant.as_str()),
        capability,
    );
    offer.provider = Some(capability_provider_ref(
        variant.component_ref(),
        observer_operation_for_variant(variant),
    ));
    Ok(capability_declaration(
        vec![offer],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
}

/// Returns the pack capability payload for an observer backend.
#[must_use]
pub fn observer_pack_capabilities(
    variant: ObserverVariant,
) -> greentic_types::CapabilitiesExtensionV1 {
    pack_capabilities_extension(
        ProviderCategory::Observer,
        variant.capability_name(),
        format!("offer.observer.{}", variant.as_str()),
        variant.component_ref(),
        observer_operation_for_variant(variant),
    )
}

/// Builds a pack manifest for an observer backend.
pub fn observer_pack_manifest(
    pack_id: PackId,
    variant: ObserverVariant,
) -> Result<PackManifest, ObserverFixtureError> {
    let provider_decl = observer_provider_decl(variant);
    let mut manifest = PackManifest {
        schema_version: "pack-v1".to_string(),
        pack_id,
        name: Some(format!("observer {}", variant.as_str())),
        version: Version::new(0, 4, 0),
        kind: PackKind::Provider,
        publisher: "greentic".to_string(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: vec![greentic_types::ComponentCapability {
            name: observer_pack_capability_id(variant.capability_name()),
            description: Some("observer capability".to_string()),
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
        .set_capabilities_extension_v1(observer_pack_capabilities(variant))
        .map_err(ObserverFixtureError::from)?;
    Ok(manifest)
}

/// Encodes an observer pack manifest to CBOR bytes.
pub fn observer_pack_manifest_cbor(
    pack_id: PackId,
    variant: ObserverVariant,
) -> Result<Vec<u8>, ObserverFixtureError> {
    let manifest = observer_pack_manifest(pack_id, variant)?;
    Ok(encode_pack_manifest(&manifest)?)
}
