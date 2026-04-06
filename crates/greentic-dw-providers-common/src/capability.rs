use crate::ProviderCategory;

use greentic_cap_types::{
    CapabilityConsume, CapabilityDeclaration, CapabilityId, CapabilityIdError, CapabilityOffer,
    CapabilityProfile, CapabilityProviderRef, CapabilityRequirement, CapabilityValidationError,
};
use greentic_types::{CapabilitiesExtensionV1, CapabilityOfferV1, CapabilityProviderRefV1};

/// Returns the canonical capability identifier for the DW capability contract.
#[must_use]
pub fn capability_uri(category: ProviderCategory, capability: impl AsRef<str>) -> String {
    crate::category::capability_uri(category, capability)
}

/// Returns the canonical pack capability identifier used in `pack.cbor` extension payloads.
#[must_use]
pub fn pack_capability_id(category: ProviderCategory, capability: impl AsRef<str>) -> String {
    format!("greentic.cap.{}.{}", category.as_str(), capability.as_ref())
}

/// Returns a validated capability identifier for the DW capability contract.
pub fn capability_id(
    category: ProviderCategory,
    capability: impl AsRef<str>,
) -> Result<CapabilityId, CapabilityIdError> {
    CapabilityId::new(capability_uri(category, capability))
}

/// Creates a provider reference for a capability declaration.
#[must_use]
pub fn capability_provider_ref(
    component_ref: impl Into<String>,
    operation: impl Into<String>,
) -> CapabilityProviderRef {
    CapabilityProviderRef {
        component_ref: component_ref.into(),
        operation: operation.into(),
        operation_map: Vec::new(),
    }
}

/// Creates a capability offer for the DW capability declaration model.
pub fn capability_offer(
    category: ProviderCategory,
    capability: impl AsRef<str>,
    offer_id: impl Into<String>,
    component_ref: impl Into<String>,
    operation: impl Into<String>,
) -> Result<CapabilityOffer, CapabilityIdError> {
    let mut offer = CapabilityOffer::new(offer_id, capability_id(category, capability)?);
    offer.provider = Some(capability_provider_ref(component_ref, operation));
    Ok(offer)
}

/// Creates a capability requirement for the DW capability declaration model.
pub fn capability_requirement(
    category: ProviderCategory,
    capability: impl AsRef<str>,
    requirement_id: impl Into<String>,
) -> Result<CapabilityRequirement, CapabilityIdError> {
    Ok(CapabilityRequirement::new(
        requirement_id,
        capability_id(category, capability)?,
    ))
}

/// Creates a capability consume entry for the DW capability declaration model.
pub fn capability_consume(
    category: ProviderCategory,
    capability: impl AsRef<str>,
    consume_id: impl Into<String>,
) -> Result<CapabilityConsume, CapabilityIdError> {
    Ok(CapabilityConsume::new(
        consume_id,
        capability_id(category, capability)?,
    ))
}

/// Creates a capability profile by id.
#[must_use]
pub fn capability_profile(id: impl Into<String>) -> CapabilityProfile {
    CapabilityProfile::new(id)
}

/// Creates a capability declaration from the supplied sections.
#[must_use]
pub fn capability_declaration(
    offers: Vec<CapabilityOffer>,
    requires: Vec<CapabilityRequirement>,
    consumes: Vec<CapabilityConsume>,
    profiles: Vec<CapabilityProfile>,
) -> CapabilityDeclaration {
    CapabilityDeclaration {
        offers,
        requires,
        consumes,
        profiles,
    }
}

/// Builds a one-off capability declaration used by the provider scaffold.
pub fn sample_capability_declaration(
    category: ProviderCategory,
    capability: impl AsRef<str>,
    offer_id: impl Into<String>,
    component_ref: impl Into<String>,
    operation: impl Into<String>,
) -> Result<CapabilityDeclaration, CapabilityIdError> {
    Ok(capability_declaration(
        vec![capability_offer(
            category,
            capability,
            offer_id,
            component_ref,
            operation,
        )?],
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
}

/// Creates a pack capability provider reference.
#[must_use]
pub fn sample_capability_provider_ref_v1(
    component_ref: impl Into<String>,
    operation: impl Into<String>,
) -> CapabilityProviderRefV1 {
    CapabilityProviderRefV1 {
        component_ref: component_ref.into(),
        op: operation.into(),
    }
}

/// Creates a pack capability offer entry for a given category and capability name.
#[must_use]
pub fn sample_capability_offer_v1(
    category: ProviderCategory,
    capability: impl AsRef<str>,
    offer_id: impl Into<String>,
    component_ref: impl Into<String>,
    operation: impl Into<String>,
) -> CapabilityOfferV1 {
    CapabilityOfferV1 {
        offer_id: offer_id.into(),
        cap_id: pack_capability_id(category, capability),
        version: "v1".to_string(),
        provider: sample_capability_provider_ref_v1(component_ref, operation),
        scope: None,
        priority: 0,
        requires_setup: false,
        setup: None,
        applies_to: None,
    }
}

/// Creates a one-off capabilities extension payload used in pack fixtures.
#[must_use]
pub fn pack_capabilities_extension(
    category: ProviderCategory,
    capability: impl AsRef<str>,
    offer_id: impl Into<String>,
    component_ref: impl Into<String>,
    operation: impl Into<String>,
) -> CapabilitiesExtensionV1 {
    CapabilitiesExtensionV1::new(vec![sample_capability_offer_v1(
        category,
        capability,
        offer_id,
        component_ref,
        operation,
    )])
}

/// Validates a capability declaration with the shared Greentic capability model.
pub fn validate_capability_declaration(
    declaration: &CapabilityDeclaration,
) -> Result<(), CapabilityValidationError> {
    declaration.validate()
}
