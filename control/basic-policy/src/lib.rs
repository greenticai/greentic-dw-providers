#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Basic policy control provider.

use greentic_dw_control::{Control, ControlDecision, ControlRequest};
use greentic_dw_providers_common::{ControlVariant, control_pack_manifest, control_provider_decl};
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use serde_json::json;

/// Basic policy evaluator using simple allow/deny heuristics.
#[derive(Default)]
pub struct BasicPolicyControl;

impl BasicPolicyControl {
    /// Creates a new basic policy controller.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Returns the canonical provider declaration for this backend.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        control_provider_decl(ControlVariant::BasicPolicy)
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.control.basic-policy")?;
        control_pack_manifest(pack_id, ControlVariant::BasicPolicy)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }
}

impl Control for BasicPolicyControl {
    fn evaluate(&self, tenant: &TenantCtx, request: ControlRequest) -> GResult<ControlDecision> {
        let denied = request
            .attributes
            .get("policy")
            .map(|value| value == "deny")
            .unwrap_or(false)
            || request.action.contains("forbidden");

        Ok(ControlDecision {
            allowed: !denied,
            policy: "basic-policy".to_string(),
            reason: if denied {
                "request matched a deny policy rule".to_string()
            } else {
                "request passed the basic allow/deny checks".to_string()
            },
            details: json!({
                "tenant_id": tenant.tenant_id.to_string(),
                "action": request.action,
                "actor": request.actor,
            }),
        })
    }

    fn guard(&self, tenant: &TenantCtx, request: ControlRequest) -> GResult<ControlDecision> {
        self.evaluate(tenant, request)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::BasicPolicyControl;
    use greentic_dw_control::{Control, ControlRequest};
    use greentic_types::{EnvId, TenantCtx, TenantId};

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-control").expect("tenant id"),
        )
    }

    #[test]
    fn basic_policy_denies_explicit_deny_rule() {
        let control = BasicPolicyControl::new();
        let decision = control
            .evaluate(
                &tenant(),
                ControlRequest::new("req-1", "read", "alice")
                    .expect("request")
                    .with_attribute("policy", "deny"),
            )
            .expect("decision");
        assert!(!decision.allowed);
    }
}
