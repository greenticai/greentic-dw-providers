use greentic_dw_delegation::{
    DelegationFanout, DelegationProvider, DelegationRequest, DelegationTarget,
};
use greentic_dw_delegation_capability_match::CapabilityMatchDelegationProvider;
use greentic_types::{EnvId, TenantCtx, TenantId};

fn tenant() -> TenantCtx {
    TenantCtx::new(
        EnvId::try_from("dev").expect("env id"),
        TenantId::try_from("tenant-delegation").expect("tenant id"),
    )
}

fn target(id: &str, capabilities: &[&str], schemas: &[&str], tags: &[&str]) -> DelegationTarget {
    let mut target = DelegationTarget::new(id).expect("target");
    target.capabilities = capabilities
        .iter()
        .map(|value| (*value).to_string())
        .collect();
    target.schemas = schemas.iter().map(|value| (*value).to_string()).collect();
    target.tags = tags.iter().map(|value| (*value).to_string()).collect();
    target
}

#[test]
fn deterministic_routing_for_identical_inputs() {
    let provider = CapabilityMatchDelegationProvider::new();
    let request = DelegationRequest::new("req-1", "plan")
        .expect("request")
        .with_required_capability("planner")
        .with_available_target(target("agent-b", &["planner"], &[], &[]))
        .with_available_target(target("agent-a", &["planner"], &[], &[]));

    let first = provider
        .delegate(&tenant(), request.clone())
        .expect("first");
    let second = provider.delegate(&tenant(), request).expect("second");
    assert_eq!(first, second);
    assert_eq!(first.targets[0].target_id, "agent-a");
}

#[test]
fn parallel_fanout_yields_explicit_target_list() {
    let provider = CapabilityMatchDelegationProvider::new();
    let request = DelegationRequest::new("req-1", "plan")
        .expect("request")
        .with_required_capability("planner")
        .with_fanout(DelegationFanout::Parallel)
        .with_available_target(target("agent-a", &["planner"], &["plan.v1"], &["safe"]))
        .with_available_target(target("agent-b", &["planner"], &["plan.v1"], &["safe"]));

    let decision = provider.delegate(&tenant(), request).expect("decision");
    assert_eq!(decision.fanout, DelegationFanout::Parallel);
    assert_eq!(decision.targets.len(), 2);
}

#[test]
fn missing_agent_returns_clean_error() {
    let provider = CapabilityMatchDelegationProvider::new();
    let request = DelegationRequest::new("req-1", "plan")
        .expect("request")
        .with_required_capability("planner");

    let err = provider
        .delegate(&tenant(), request)
        .expect_err("missing agent");
    assert!(err.to_string().contains("no available agent"));
}

#[test]
fn control_denied_route_remains_denied() {
    let provider = CapabilityMatchDelegationProvider::new();
    let request = DelegationRequest::new("req-1", "plan")
        .expect("request")
        .with_control_allowed(false)
        .with_required_capability("planner")
        .with_available_target(target("agent-a", &["planner"], &[], &[]));

    let err = provider.delegate(&tenant(), request).expect_err("denied");
    assert!(err.to_string().contains("control policy"));
}
