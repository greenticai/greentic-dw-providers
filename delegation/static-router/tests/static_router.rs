use greentic_dw_delegation::{
    DelegationFanout, DelegationProvider, DelegationRequest, DelegationTarget,
};
use greentic_dw_delegation_static_router::{
    StaticRouteRule, StaticRouterConfig, StaticRouterDelegationProvider,
};
use greentic_types::{EnvId, TenantCtx, TenantId};

fn tenant() -> TenantCtx {
    TenantCtx::new(
        EnvId::try_from("dev").expect("env id"),
        TenantId::try_from("tenant-delegation").expect("tenant id"),
    )
}

#[test]
fn deterministic_routing_for_identical_inputs() {
    let mut route = StaticRouteRule::new(vec!["agent-a".to_string()]).expect("route");
    route.step_kind = Some("plan".to_string());
    let provider = StaticRouterDelegationProvider::new(StaticRouterConfig {
        routes: vec![route],
    });
    let request = DelegationRequest::new("req-1", "plan")
        .expect("request")
        .with_available_target(DelegationTarget::new("agent-a").expect("agent-a"));

    let first = provider
        .delegate(&tenant(), request.clone())
        .expect("first");
    let second = provider.delegate(&tenant(), request).expect("second");
    assert_eq!(first, second);
    assert_eq!(first.targets[0].target_id, "agent-a");
}

#[test]
fn parallel_fanout_yields_explicit_target_list() {
    let mut route =
        StaticRouteRule::new(vec!["agent-a".to_string(), "agent-b".to_string()]).expect("route");
    route.step_kind = Some("review".to_string());
    route.fanout = DelegationFanout::Parallel;
    let provider = StaticRouterDelegationProvider::new(StaticRouterConfig {
        routes: vec![route],
    });
    let request = DelegationRequest::new("req-1", "review")
        .expect("request")
        .with_fanout(DelegationFanout::Parallel)
        .with_available_target(DelegationTarget::new("agent-a").expect("agent-a"))
        .with_available_target(DelegationTarget::new("agent-b").expect("agent-b"));

    let decision = provider.delegate(&tenant(), request).expect("decision");
    assert_eq!(decision.fanout, DelegationFanout::Parallel);
    assert_eq!(decision.targets.len(), 2);
}

#[test]
fn missing_agent_returns_clean_error() {
    let mut route = StaticRouteRule::new(vec!["agent-missing".to_string()]).expect("route");
    route.step_kind = Some("plan".to_string());
    let provider = StaticRouterDelegationProvider::new(StaticRouterConfig {
        routes: vec![route],
    });
    let request = DelegationRequest::new("req-1", "plan").expect("request");

    let err = provider
        .delegate(&tenant(), request)
        .expect_err("missing agent");
    assert!(err.to_string().contains("unavailable"));
}

#[test]
fn control_denied_route_remains_denied() {
    let mut route = StaticRouteRule::new(vec!["agent-a".to_string()]).expect("route");
    route.step_kind = Some("plan".to_string());
    let provider = StaticRouterDelegationProvider::new(StaticRouterConfig {
        routes: vec![route],
    });
    let request = DelegationRequest::new("req-1", "plan")
        .expect("request")
        .with_control_allowed(false)
        .with_available_target(DelegationTarget::new("agent-a").expect("agent-a"));

    let err = provider.delegate(&tenant(), request).expect_err("denied");
    assert!(err.to_string().contains("control policy"));
}
