#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Router-lite engine provider.

use greentic_dw_engine::{Engine, EngineDecision, EngineRequest};
use greentic_dw_providers_common::{EngineVariant, engine_pack_manifest, engine_provider_decl};
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use serde_json::json;

/// Lightweight router engine with simple heuristics for candidate selection.
#[derive(Default)]
pub struct RouterLiteEngine;

impl RouterLiteEngine {
    /// Creates a new router-lite engine.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Returns the canonical provider declaration for this backend.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        engine_provider_decl(EngineVariant::RouterLite)
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.engine.router-lite")?;
        engine_pack_manifest(pack_id, EngineVariant::RouterLite)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }

    fn route_candidate(request: &EngineRequest) -> String {
        if let Some(candidate) = request
            .candidates
            .iter()
            .find(|candidate| candidate.contains("router") || candidate.contains("plan"))
        {
            return candidate.clone();
        }
        if let Some(candidate) = request
            .candidates
            .iter()
            .find(|candidate| candidate.contains("safe") || candidate.contains("audit"))
        {
            return candidate.clone();
        }
        request
            .candidates
            .first()
            .cloned()
            .unwrap_or_else(|| format!("route:{}", request.goal.replace(' ', "-")))
    }
}

impl Engine for RouterLiteEngine {
    fn decide(&self, tenant: &TenantCtx, request: EngineRequest) -> GResult<EngineDecision> {
        let selected = Self::route_candidate(&request);
        Ok(EngineDecision {
            decision_type: "router-lite-decide".to_string(),
            selected: selected.clone(),
            reason: "ranked candidates using lightweight route heuristics".to_string(),
            plan: vec![
                "classify request".to_string(),
                format!("route to {selected}"),
                "execute selected route".to_string(),
            ],
            details: json!({
                "tenant_id": tenant.tenant_id.to_string(),
                "goal": request.goal,
                "candidate_count": request.candidates.len(),
            }),
        })
    }

    fn route(&self, tenant: &TenantCtx, request: EngineRequest) -> GResult<EngineDecision> {
        let mut decision = self.decide(tenant, request)?;
        decision.decision_type = "router-lite-route".to_string();
        Ok(decision)
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::RouterLiteEngine;
    use greentic_dw_engine::{Engine, EngineRequest};
    use greentic_types::{EnvId, TenantCtx, TenantId};

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-router").expect("tenant id"),
        )
    }

    #[test]
    fn router_lite_prefers_router_candidate() {
        let engine = RouterLiteEngine::new();
        let decision = engine
            .route(
                &tenant(),
                EngineRequest::new("req-1", "route task")
                    .expect("request")
                    .with_candidate("route.safe")
                    .with_candidate("route.router-lite"),
            )
            .expect("decision");

        assert_eq!(decision.selected, "route.router-lite");
    }
}
