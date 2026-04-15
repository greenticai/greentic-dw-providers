use greentic_dw_planning::{PlanDocument, PlanRequest, PlanStep, PlanningProvider};
use greentic_dw_planning_static::{StaticPlanningConfig, StaticPlanningProvider};
use greentic_types::{EnvId, TenantCtx, TenantId};
use serde_json::json;

fn tenant() -> TenantCtx {
    TenantCtx::new(
        EnvId::try_from("dev").expect("env id"),
        TenantId::try_from("tenant-planning").expect("tenant id"),
    )
}

#[test]
fn static_returns_exact_expected_plan() {
    let plan = PlanDocument::new("plan-static")
        .expect("plan")
        .with_step(
            PlanStep::new("step-1", "research")
                .expect("step")
                .with_payload(json!({"topic": "planning"})),
        )
        .with_step(
            PlanStep::new("step-2", "draft")
                .expect("step")
                .with_dependency("step-1")
                .with_payload(json!({"style": "strict"})),
        );
    let provider = StaticPlanningProvider::new(StaticPlanningConfig {
        plan: plan.clone(),
        interpolate_variables: false,
    })
    .expect("provider");

    let actual = provider
        .plan(
            &tenant(),
            PlanRequest::new("req-1", "produce a plan").expect("request"),
        )
        .expect("plan");

    assert_eq!(actual, plan);
}

#[test]
fn static_can_interpolate_context_variables() {
    let plan = PlanDocument::new("plan-{{request_id}}")
        .expect("plan")
        .with_step(
            PlanStep::new("step-1", "research")
                .expect("step")
                .with_payload(json!({"topic": "{{topic}}", "goal": "{{goal}}"})),
        );
    let provider = StaticPlanningProvider::new(StaticPlanningConfig {
        plan,
        interpolate_variables: true,
    })
    .expect("provider");

    let actual = provider
        .plan(
            &tenant(),
            PlanRequest::new("req-2", "map the topic")
                .expect("request")
                .with_context(json!({"topic": "cargo"})),
        )
        .expect("plan");

    assert_eq!(actual.plan_id, "plan-req-2");
    assert_eq!(actual.steps[0].payload["topic"], json!("cargo"));
    assert_eq!(actual.steps[0].payload["goal"], json!("map the topic"));
}
