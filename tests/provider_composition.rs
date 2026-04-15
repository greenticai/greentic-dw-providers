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
use greentic_dw_planning::{PlanDocument, PlanStep, PlanningProvider};
use greentic_dw_planning_static::{StaticPlanningConfig, StaticPlanningProvider};
use greentic_dw_reflection::{ReflectionProvider, ReviewDisposition, ReviewRequest};
use greentic_dw_reflection_schema_check::SchemaCheckReflectionProvider;
use greentic_dw_workspace::{
    WorkspaceArtifactId, WorkspaceArtifactVersion, WorkspaceProvider, WorkspaceScope,
};
use greentic_dw_workspace_in_memory::InMemoryWorkspaceProvider;
use greentic_types::{EnvId, GResult, GreenticError, TenantCtx, TenantId};
use serde_json::json;

fn tenant() -> TenantCtx {
    TenantCtx::new(
        EnvId::try_from("dev").expect("env id"),
        TenantId::try_from("tenant-compose").expect("tenant id"),
    )
}

struct CompositionHarness {
    planner: StaticPlanningProvider,
    static_context: StaticContextProvider,
    workspace_enabled: bool,
    reflection: Option<SchemaCheckReflectionProvider>,
    delegation: Option<StaticRouterDelegationProvider>,
}

impl CompositionHarness {
    fn run(&self, tenant: &TenantCtx) -> GResult<ComposedRun> {
        let request = ContextRequest::new("req-compose")
            .expect("request")
            .with_runtime_metadata(json!({"stage": "compose"}));
        let plan = self.planner.plan(
            tenant,
            greentic_dw_planning::PlanRequest::new("req-compose", "compose providers")
                .expect("plan request"),
        )?;

        let static_package = self.static_context.assemble(tenant, request)?;
        let mut retrieved_package = ContextPackage::empty();

        if self.workspace_enabled {
            let workspace = InMemoryWorkspaceProvider::new();
            let scope = WorkspaceScope::new("analysis")?;
            let artifact = WorkspaceArtifactId::new("plan-doc")?;
            workspace.write_version(
                tenant,
                &scope,
                &artifact,
                WorkspaceArtifactVersion::new(
                    "v1",
                    json!({
                        "plan_id": plan.plan_id,
                        "step_count": plan.steps.len(),
                    }),
                )?,
            )?;
            let retrieval = RetrievalContextProvider::new(
                workspace,
                vec![
                    RetrievalRef::Workspace {
                        scope,
                        artifact_id: artifact,
                        score: 0.95,
                    },
                    RetrievalRef::PlanStep {
                        plan: plan.clone(),
                        step_id: "step-1".to_string(),
                        score: 0.6,
                    },
                ],
            );
            retrieved_package =
                retrieval.assemble(tenant, ContextRequest::new("req-retrieval")?)?;
        }

        let mut combined = ContextPackage::empty();
        for fragment in static_package.fragments {
            combined = combined.with_fragment(fragment);
        }
        for fragment in retrieved_package.fragments {
            combined = combined.with_fragment(fragment);
        }

        let compressed = CompressorContextProvider::new(
            combined,
            CompressorConfig {
                max_fragments: 3,
                summarizer_configured: true,
            },
        )
        .assemble(tenant, ContextRequest::new("req-compress")?)?;

        if let Some(reflection) = &self.reflection {
            let outcome = reflection.review(
                tenant,
                ReviewRequest::new(
                    "req-review",
                    json!({
                        "title": "Composed package",
                        "fragments": compressed.fragments.iter().map(|fragment| fragment.id.clone()).collect::<Vec<_>>(),
                    }),
                )?
                .with_schema(json!({
                    "type": "object",
                    "required": ["title", "fragments"],
                    "properties": {
                        "title": { "type": "string" },
                        "fragments": { "type": "array" }
                    }
                })),
            )?;
            if outcome.disposition != ReviewDisposition::Accept {
                return Err(GreenticError::new(
                    greentic_types::ErrorCode::InvalidInput,
                    format!(
                        "reflection rejected composed output: {:?}",
                        outcome.findings
                    ),
                ));
            }
        }

        Ok(ComposedRun {
            plan,
            context: compressed,
        })
    }

    fn delegate_step(&self, tenant: &TenantCtx, step: &str) -> GResult<Vec<String>> {
        let Some(delegation) = &self.delegation else {
            return Err(GreenticError::new(
                greentic_types::ErrorCode::InvalidInput,
                "delegation provider missing for delegatable step",
            ));
        };
        let decision = delegation.delegate(
            tenant,
            DelegationRequest::new("req-delegate", step)?
                .with_goal("delegate work")
                .with_fanout(DelegationFanout::Single)
                .with_available_target(DelegationTarget::new("agent-a")?),
        )?;
        Ok(decision
            .targets
            .into_iter()
            .map(|target| target.target_id)
            .collect())
    }

    fn enforce_reflection_policy(&self) -> GResult<()> {
        if self.reflection.is_none() {
            return Err(GreenticError::new(
                greentic_types::ErrorCode::InvalidInput,
                "mandatory reflection policy requires a reflection provider",
            ));
        }
        Ok(())
    }
}

struct ComposedRun {
    plan: PlanDocument,
    context: ContextPackage,
}

fn harness_with(workspace: bool, reflection: bool, delegation: bool) -> CompositionHarness {
    let plan = PlanDocument::new("plan-compose")
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
        );
    let planner = StaticPlanningProvider::new(StaticPlanningConfig {
        plan,
        interpolate_variables: false,
    })
    .expect("planner");

    let static_context = StaticContextProvider::new(StaticContextConfig {
        fragments: vec![
            ContextFragment::new("intro", json!({"text": "system prompt"}))
                .expect("intro")
                .with_source_kind(ContextSourceKind::Static)
                .with_score(0.9)
                .with_pinned(true),
            ContextFragment::new("policy", json!({"text": "follow constraints"}))
                .expect("policy")
                .with_source_kind(ContextSourceKind::Static)
                .with_score(0.8),
        ],
        include_runtime_metadata: true,
    });

    let delegation_provider = if delegation {
        let mut route = StaticRouteRule::new(vec!["agent-a".to_string()]).expect("route");
        route.step_kind = Some("delegate".to_string());
        Some(StaticRouterDelegationProvider::new(StaticRouterConfig {
            routes: vec![route],
        }))
    } else {
        None
    };

    CompositionHarness {
        planner,
        static_context,
        workspace_enabled: workspace,
        reflection: reflection.then(SchemaCheckReflectionProvider::new),
        delegation: delegation_provider,
    }
}

#[test]
fn planning_plus_context_only() {
    let harness = harness_with(false, false, false);
    let run = harness.run(&tenant()).expect("run");
    assert_eq!(run.plan.steps.len(), 2);
    assert!(run.context.fragments.len() >= 2);
}

#[test]
fn planning_context_and_workspace() {
    let harness = harness_with(true, false, false);
    let run = harness.run(&tenant()).expect("run");
    assert!(
        run.context
            .fragments
            .iter()
            .any(|fragment| fragment.source_kind == ContextSourceKind::Workspace)
    );
}

#[test]
fn planning_context_and_reflection() {
    let harness = harness_with(false, true, false);
    let run = harness.run(&tenant()).expect("run");
    assert_eq!(run.plan.plan_id, "plan-compose");
    assert!(run.context.metadata["provider"].is_string());
}

#[test]
fn full_stack_all_five_families() {
    let harness = harness_with(true, true, true);
    let run = harness.run(&tenant()).expect("run");
    let delegated = harness
        .delegate_step(&tenant(), "delegate")
        .expect("delegated");
    assert_eq!(run.plan.steps.len(), 2);
    assert_eq!(delegated, vec!["agent-a".to_string()]);
}

#[test]
fn delegate_step_without_delegation_provider_rejected() {
    let harness = harness_with(true, true, false);
    let err = harness
        .delegate_step(&tenant(), "delegate")
        .expect_err("missing delegation");
    assert!(err.to_string().contains("delegation provider missing"));
}

#[test]
fn mandatory_reflection_policy_without_reflection_provider_rejected() {
    let harness = harness_with(true, false, true);
    let err = harness
        .enforce_reflection_policy()
        .expect_err("missing reflection");
    assert!(
        err.to_string()
            .contains("mandatory reflection policy requires a reflection provider")
    );
}
