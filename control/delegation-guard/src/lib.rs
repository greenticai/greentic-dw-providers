#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Delegation guard control provider.

use greentic_dw_control::{Control, ControlDecision, ControlRequest};
use greentic_dw_providers_common::{ControlVariant, control_pack_manifest, control_provider_decl};
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use serde_json::json;

/// Delegation guard that blocks delegation unless explicitly allowed.
#[derive(Default)]
pub struct DelegationGuardControl;

impl DelegationGuardControl {
    /// Creates a new delegation guard controller.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Returns the canonical provider declaration for this backend.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        control_provider_decl(ControlVariant::DelegationGuard)
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.control.delegation-guard")?;
        control_pack_manifest(pack_id, ControlVariant::DelegationGuard)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }
}

impl Control for DelegationGuardControl {
    fn evaluate(&self, tenant: &TenantCtx, request: ControlRequest) -> GResult<ControlDecision> {
        self.guard(tenant, request)
    }

    fn guard(&self, tenant: &TenantCtx, request: ControlRequest) -> GResult<ControlDecision> {
        let delegation_allowed = request
            .attributes
            .get("allow_delegation")
            .map(|value| value == "true")
            .unwrap_or(false);

        let allowed = !request.delegation_requested || delegation_allowed;
        Ok(ControlDecision {
            allowed,
            policy: "delegation-guard".to_string(),
            reason: if allowed {
                "delegation was not requested or was explicitly allowed".to_string()
            } else {
                "delegation requires allow_delegation=true".to_string()
            },
            details: json!({
                "tenant_id": tenant.tenant_id.to_string(),
                "delegation_requested": request.delegation_requested,
                "delegation_allowed": delegation_allowed,
                "actor": request.actor,
            }),
        })
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::DelegationGuardControl;
    use greentic_dw_control::{Control, ControlRequest};
    use greentic_types::{EnvId, TenantCtx, TenantId};

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-guard").expect("tenant id"),
        )
    }

    #[test]
    fn delegation_guard_blocks_unapproved_delegation() {
        let control = DelegationGuardControl::new();
        let decision = control
            .guard(
                &tenant(),
                ControlRequest::new("req-1", "delegate", "alice")
                    .expect("request")
                    .with_delegation_requested(true),
            )
            .expect("decision");
        assert!(!decision.allowed);
    }
}
