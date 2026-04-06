use semver::Version;
use thiserror::Error;

use crate::provider::ProviderDeclSpec;
use crate::{
    CapabilityDeclaration, ProviderCategory, capability_declaration, capability_id,
    pack_capabilities_extension, provider_decl, provider_extension_inline,
};

use greentic_types::{
    CapabilitiesExtensionError, CborError, PackId, PackKind, PackManifest, PackSignatures,
    ProviderDecl, ProviderExtensionInline, encode_pack_manifest,
};

/// Errors produced while building sample provider fixtures.
#[derive(Debug, Error)]
pub enum FixtureError {
    /// Capability identifier parsing failed.
    #[error(transparent)]
    CapabilityId(#[from] greentic_cap_types::CapabilityIdError),
    /// Pack capability extension construction failed.
    #[error(transparent)]
    CapabilitiesExtension(#[from] CapabilitiesExtensionError),
    /// Pack CBOR encoding failed.
    #[error(transparent)]
    Cbor(#[from] CborError),
}

fn default_component_ref(category: ProviderCategory, provider_name: &str) -> String {
    format!("component:dw.{}.{}", category.as_str(), provider_name)
}

fn default_export_name() -> String {
    "greentic_provider".to_string()
}

fn default_world_name() -> String {
    "greentic:provider/runtime".to_string()
}

fn sample_pack_version() -> Version {
    Version::new(0, 4, 0)
}

/// Creates a sample provider declaration for pack extension payloads.
#[must_use]
fn sample_provider_decl_with_capability_name(
    category: ProviderCategory,
    provider_name: impl AsRef<str>,
    pack_capability_name: impl AsRef<str>,
) -> ProviderDecl {
    let provider_name = provider_name.as_ref();
    provider_decl(ProviderDeclSpec {
        category,
        provider_name: provider_name.to_string(),
        capabilities: vec![pack_capability_name.as_ref().to_string()],
        ops: vec!["invoke".to_string()],
        config_schema_ref: format!("schemas/{provider_name}.json"),
        state_schema_ref: None,
        component_ref: default_component_ref(category, provider_name),
        export: default_export_name(),
        world: default_world_name(),
        docs_ref: Some(format!("docs/providers/{provider_name}.md")),
    })
}

/// Creates a sample provider declaration for pack extension payloads.
#[must_use]
pub fn sample_provider_decl(
    category: ProviderCategory,
    provider_name: impl AsRef<str>,
    capability_name: impl AsRef<str>,
) -> ProviderDecl {
    let capability_name = capability_name.as_ref();
    let pack_capability_name = pack_capability_extension_name(category, capability_name);

    sample_provider_decl_with_capability_name(category, provider_name, &pack_capability_name)
}

fn pack_capability_extension_name(category: ProviderCategory, capability_name: &str) -> String {
    format!("greentic.cap.{}.{}", category.as_str(), capability_name)
}

/// Creates a sample pack manifest with both provider and capability extension payloads.
pub fn sample_pack_manifest(
    pack_id: PackId,
    category: ProviderCategory,
    provider_name: impl AsRef<str>,
    capability_name: impl AsRef<str>,
) -> Result<PackManifest, FixtureError> {
    let provider_name = provider_name.as_ref();
    let capability_name = capability_name.as_ref();
    let pack_capability_name = pack_capability_extension_name(category, capability_name);
    let provider_decl =
        sample_provider_decl_with_capability_name(category, provider_name, &pack_capability_name);
    let mut manifest = PackManifest {
        schema_version: "pack-v1".to_string(),
        pack_id,
        name: Some(format!("{} {} pack", category.as_str(), provider_name)),
        version: sample_pack_version(),
        kind: PackKind::Provider,
        publisher: "greentic".to_string(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: vec![greentic_types::ComponentCapability {
            name: pack_capability_name,
            description: Some("sample pack capability".to_string()),
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

    manifest.set_capabilities_extension_v1(pack_capabilities_extension(
        category,
        capability_name,
        format!("offer.{provider_name}.{capability_name}"),
        default_component_ref(category, provider_name),
        default_export_name(),
    ))?;

    Ok(manifest)
}

/// Encodes a sample pack manifest to CBOR bytes.
pub fn sample_pack_manifest_cbor(
    pack_id: PackId,
    category: ProviderCategory,
    provider_name: impl AsRef<str>,
    capability_name: impl AsRef<str>,
) -> Result<Vec<u8>, FixtureError> {
    let manifest = sample_pack_manifest(pack_id, category, provider_name, capability_name)?;
    Ok(encode_pack_manifest(&manifest)?)
}

/// Returns a provider extension payload in inline form.
#[must_use]
pub fn sample_provider_extension_inline(
    category: ProviderCategory,
    provider_name: impl AsRef<str>,
    capability_name: impl AsRef<str>,
) -> ProviderExtensionInline {
    provider_extension_inline(vec![sample_provider_decl(
        category,
        provider_name,
        capability_name,
    )])
}

/// Builds a self-contained capability declaration and returns the shared model.
pub fn sample_capability_declaration_fixture(
    category: ProviderCategory,
    capability_name: impl AsRef<str>,
    offer_id: impl Into<String>,
    provider_component_ref: impl Into<String>,
    provider_operation: impl Into<String>,
) -> Result<CapabilityDeclaration, FixtureError> {
    Ok(capability_declaration(
        vec![
            capability_id(category, capability_name.as_ref())?.into_offer(
                offer_id,
                provider_component_ref,
                provider_operation,
            ),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
}

trait CapabilityIdExt {
    fn into_offer(
        self,
        offer_id: impl Into<String>,
        provider_component_ref: impl Into<String>,
        provider_operation: impl Into<String>,
    ) -> greentic_cap_types::CapabilityOffer;
}

impl CapabilityIdExt for greentic_cap_types::CapabilityId {
    fn into_offer(
        self,
        offer_id: impl Into<String>,
        provider_component_ref: impl Into<String>,
        provider_operation: impl Into<String>,
    ) -> greentic_cap_types::CapabilityOffer {
        let mut offer = greentic_cap_types::CapabilityOffer::new(offer_id, self);
        offer.provider = Some(greentic_cap_types::CapabilityProviderRef {
            component_ref: provider_component_ref.into(),
            operation: provider_operation.into(),
            operation_map: Vec::new(),
        });
        offer
    }
}
