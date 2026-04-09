#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Default engine provider.

use greentic_dw_engine::{Engine, EngineDecision, EngineRequest};
use greentic_dw_providers_common::{EngineVariant, engine_pack_manifest, engine_provider_decl};
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use serde_json::json;

/// Simple default engine that picks the first available candidate or goal-derived action.
#[derive(Default)]
pub struct DefaultEngine;

impl DefaultEngine {
    /// Creates a new default engine.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Returns the canonical provider declaration for this backend.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        engine_provider_decl(EngineVariant::Default)
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.engine.default")?;
        engine_pack_manifest(pack_id, EngineVariant::Default)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }
}

impl Engine for DefaultEngine {
    fn decide(&self, tenant: &TenantCtx, request: EngineRequest) -> GResult<EngineDecision> {
        let selected = request
            .candidates
            .first()
            .cloned()
            .unwrap_or_else(|| format!("action:{}", request.goal.replace(' ', "-")));

        Ok(EngineDecision {
            decision_type: "default".to_string(),
            selected,
            reason: "selected the first available candidate or a direct goal-derived action"
                .to_string(),
            plan: vec![
                "validate input".to_string(),
                "select direct action".to_string(),
                "execute".to_string(),
            ],
            details: json!({
                "tenant_id": tenant.tenant_id.to_string(),
                "priority": request.priority,
                "candidate_count": request.candidates.len(),
            }),
        })
    }

    fn route(&self, tenant: &TenantCtx, request: EngineRequest) -> GResult<EngineDecision> {
        self.decide(tenant, request)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::DefaultEngine;
    use greentic_dw_engine::{Engine, EngineRequest};
    use greentic_types::{EnvId, TenantCtx, TenantId};

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-engine").expect("tenant id"),
        )
    }

    #[test]
    fn default_engine_prefers_first_candidate() {
        let engine = DefaultEngine::new();
        let decision = engine
            .decide(
                &tenant(),
                EngineRequest::new("req-1", "answer user")
                    .expect("request")
                    .with_candidate("route.fast")
                    .with_candidate("route.safe"),
            )
            .expect("decision");

        assert_eq!(decision.selected, "route.fast");
    }
}
