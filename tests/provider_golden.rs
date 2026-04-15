use greentic_dw_context::{
    ContextFragment, ContextPackage, ContextProvider, ContextRequest, ContextSourceKind,
};
use greentic_dw_context_compressor::{CompressorConfig, CompressorContextProvider};
use greentic_dw_context_retrieval::{RetrievalContextProvider, RetrievalRef};
use greentic_dw_context_static::{StaticContextConfig, StaticContextProvider};
use greentic_dw_delegation::{
    DelegationFanout, DelegationProvider, DelegationRequest, DelegationTarget,
};
use greentic_dw_delegation_static_router::{
    StaticRouteRule, StaticRouterConfig, StaticRouterDelegationProvider,
};
use greentic_dw_planning::{PlanDocument, PlanRequest, PlanStep, PlanningProvider};
use greentic_dw_planning_static::{StaticPlanningConfig, StaticPlanningProvider};
use greentic_dw_reflection::{ReflectionProvider, ReviewRequest};
use greentic_dw_reflection_schema_check::SchemaCheckReflectionProvider;
use greentic_dw_workspace::{
    WorkspaceArtifactId, WorkspaceArtifactVersion, WorkspaceProvider, WorkspaceScope,
};
use greentic_dw_workspace_in_memory::InMemoryWorkspaceProvider;
use greentic_types::{EnvId, TenantCtx, TenantId};
use serde::Serialize;
use serde_json::json;

fn tenant() -> TenantCtx {
    TenantCtx::new(
        EnvId::try_from("dev").expect("env id"),
        TenantId::try_from("tenant-golden").expect("tenant id"),
    )
}

#[test]
fn plan_fixture_matches_golden() {
    let planner = StaticPlanningProvider::new(StaticPlanningConfig {
        plan: PlanDocument::new("plan-golden")
            .expect("plan")
            .with_step(
                PlanStep::new("step-1", "research")
                    .expect("step-1")
                    .with_payload(json!({
                        "topic": "golden testing",
                        "audience": "reviewers"
                    })),
            )
            .with_step(
                PlanStep::new("step-2", "draft")
                    .expect("step-2")
                    .with_dependency("step-1")
                    .with_payload(json!({
                        "style": "deterministic",
                        "format": "json"
                    })),
            ),
        interpolate_variables: false,
    })
    .expect("planner");

    let plan = planner
        .plan(
            &tenant(),
            PlanRequest::new("req-plan-golden", "create fixtures").expect("request"),
        )
        .expect("plan");

    assert_json_golden(&plan, include_str!("golden/plan_document.json"));
}

#[test]
fn workspace_history_matches_golden() {
    let provider = InMemoryWorkspaceProvider::new();
    let tenant = tenant();
    let scope = WorkspaceScope::new("analysis").expect("scope");
    let artifact = WorkspaceArtifactId::new("plan-doc").expect("artifact");

    provider
        .write_version(
            &tenant,
            &scope,
            &artifact,
            WorkspaceArtifactVersion::new(
                "v1",
                json!({
                    "plan_id": "plan-golden",
                    "status": "draft"
                }),
            )
            .expect("v1")
            .with_provenance(None::<String>, Some("planning/static"))
            .with_attribute("phase", json!("draft")),
        )
        .expect("write v1");
    provider
        .write_version(
            &tenant,
            &scope,
            &artifact,
            WorkspaceArtifactVersion::new(
                "v2",
                json!({
                    "plan_id": "plan-golden",
                    "status": "approved",
                    "reviewed": true
                }),
            )
            .expect("v2")
            .with_provenance(Some("v1"), Some("reflection/schema-check"))
            .with_attribute("phase", json!("approved")),
        )
        .expect("write v2");

    let history = provider
        .load_history(&tenant, &scope, &artifact)
        .expect("history");

    assert_json_golden(&history, include_str!("golden/workspace_history.json"));
}

#[test]
fn delegation_decision_matches_golden() {
    let mut route = StaticRouteRule::new(vec!["agent-blue".to_string(), "agent-green".to_string()])
        .expect("route");
    route.step_kind = Some("delegate".to_string());
    route.output_schema = Some("plan.fragment".to_string());
    route.tags = vec!["research".to_string(), "parallel".to_string()];
    route.goal_patterns = vec!["collect evidence".to_string()];
    route.fanout = DelegationFanout::Parallel;

    let router = StaticRouterDelegationProvider::new(StaticRouterConfig {
        routes: vec![route],
    });

    let decision = router
        .delegate(
            &tenant(),
            DelegationRequest::new("req-route-golden", "delegate")
                .expect("request")
                .with_goal("collect evidence for review")
                .with_output_schema("plan.fragment")
                .with_tag("research")
                .with_tag("parallel")
                .with_available_target(
                    DelegationTarget::new("agent-green")
                        .expect("green")
                        .with_capabilities(["search", "summarize"])
                        .with_schemas(["plan.fragment"])
                        .with_tags(["research"]),
                )
                .with_available_target(
                    DelegationTarget::new("agent-blue")
                        .expect("blue")
                        .with_capabilities(["search", "verify"])
                        .with_schemas(["plan.fragment"])
                        .with_tags(["parallel", "research"]),
                )
                .with_fanout(DelegationFanout::Parallel),
        )
        .expect("decision");

    assert_json_golden(&decision, include_str!("golden/delegation_decision.json"));
}

#[test]
fn reflection_outcome_matches_golden() {
    let provider = SchemaCheckReflectionProvider::new();
    let outcome = provider
        .review(
            &tenant(),
            ReviewRequest::new(
                "req-review-golden",
                json!({
                    "title": "Golden package"
                }),
            )
            .expect("request")
            .with_schema(json!({
                "type": "object",
                "required": ["title", "fragments"],
                "properties": {
                    "title": { "type": "string" },
                    "fragments": {
                        "type": "array",
                        "items": { "type": "string" }
                    }
                }
            })),
        )
        .expect("outcome");

    assert_json_golden(&outcome, include_str!("golden/reflection_outcome.json"));
}

#[test]
fn composed_context_matches_golden() {
    let tenant = tenant();
    let plan = fixture_plan();
    let static_context = StaticContextProvider::new(StaticContextConfig {
        fragments: vec![
            ContextFragment::new("intro", json!({"text": "system prompt"}))
                .expect("intro")
                .with_source_kind(ContextSourceKind::Static)
                .with_provenance_ref("static/intro")
                .with_score(0.9)
                .with_pinned(true),
            ContextFragment::new("policy", json!({"text": "follow constraints"}))
                .expect("policy")
                .with_source_kind(ContextSourceKind::Static)
                .with_provenance_ref("static/policy")
                .with_score(0.8),
        ],
        include_runtime_metadata: true,
    });
    let static_package = static_context
        .assemble(
            &tenant,
            ContextRequest::new("req-static-golden")
                .expect("request")
                .with_runtime_metadata(json!({"stage": "golden"})),
        )
        .expect("static package");

    let workspace = InMemoryWorkspaceProvider::new();
    let scope = WorkspaceScope::new("analysis").expect("scope");
    let artifact = WorkspaceArtifactId::new("plan-doc").expect("artifact");
    workspace
        .write_version(
            &tenant,
            &scope,
            &artifact,
            WorkspaceArtifactVersion::new(
                "v1",
                json!({
                    "plan_id": plan.plan_id,
                    "step_count": plan.steps.len()
                }),
            )
            .expect("workspace version"),
        )
        .expect("workspace write");

    let retrieval = RetrievalContextProvider::new(
        workspace,
        vec![
            RetrievalRef::Workspace {
                scope,
                artifact_id: artifact,
                score: 0.95,
            },
            RetrievalRef::PlanStep {
                plan,
                step_id: "step-1".to_string(),
                score: 0.6,
            },
            RetrievalRef::MemoryRef {
                reference: "constraints".to_string(),
                content: json!({"items": ["be concise", "be deterministic"]}),
                score: 0.4,
            },
        ],
    );
    let retrieved_package = retrieval
        .assemble(
            &tenant,
            ContextRequest::new("req-retrieval-golden").expect("request"),
        )
        .expect("retrieved package");

    let combined = combine_packages(static_package, retrieved_package);
    let compressed = CompressorContextProvider::new(
        combined,
        CompressorConfig {
            max_fragments: 3,
            summarizer_configured: true,
        },
    )
    .assemble(
        &tenant,
        ContextRequest::new("req-compress-golden").expect("request"),
    )
    .expect("compressed package");

    assert_json_golden(&compressed, include_str!("golden/context_package.json"));
}

fn fixture_plan() -> PlanDocument {
    PlanDocument::new("plan-compose")
        .expect("plan")
        .with_step(
            PlanStep::new("step-1", "research")
                .expect("step")
                .with_payload(json!({"topic": "composition"})),
        )
        .with_step(
            PlanStep::new("step-2", "draft")
                .expect("step")
                .with_dependency("step-1")
                .with_payload(json!({"mode": "strict"})),
        )
}

fn combine_packages(left: ContextPackage, right: ContextPackage) -> ContextPackage {
    let mut combined = ContextPackage::empty();
    for fragment in left.fragments {
        combined = combined.with_fragment(fragment);
    }
    for fragment in right.fragments {
        combined = combined.with_fragment(fragment);
    }
    combined
}

fn assert_json_golden<T: Serialize>(actual: &T, expected: &str) {
    let actual = serde_json::to_string_pretty(actual).expect("serialize");
    assert_eq!(format!("{actual}\n"), expected);
}

trait DelegationTargetFixtureExt {
    fn with_capabilities<const N: usize>(self, capabilities: [&str; N]) -> Self;
    fn with_schemas<const N: usize>(self, schemas: [&str; N]) -> Self;
    fn with_tags<const N: usize>(self, tags: [&str; N]) -> Self;
}

impl DelegationTargetFixtureExt for DelegationTarget {
    fn with_capabilities<const N: usize>(mut self, capabilities: [&str; N]) -> Self {
        self.capabilities = capabilities.into_iter().map(str::to_string).collect();
        self
    }

    fn with_schemas<const N: usize>(mut self, schemas: [&str; N]) -> Self {
        self.schemas = schemas.into_iter().map(str::to_string).collect();
        self
    }

    fn with_tags<const N: usize>(mut self, tags: [&str; N]) -> Self {
        self.tags = tags.into_iter().map(str::to_string).collect();
        self
    }
}
