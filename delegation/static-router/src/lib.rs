#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Deterministic static delegation router.

use greentic_dw_delegation::{
    DelegationDecision, DelegationFanout, DelegationProvider, DelegationRequest, DelegationTarget,
};
use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};

/// Static route entry matched against a delegation request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticRouteRule {
    /// Optional step kind filter.
    pub step_kind: Option<String>,
    /// Optional required schema.
    pub output_schema: Option<String>,
    /// Required tags that must all be present.
    pub tags: Vec<String>,
    /// Optional goal substring patterns.
    pub goal_patterns: Vec<String>,
    /// Ordered explicit target ids.
    pub target_ids: Vec<String>,
    /// Selected fanout behavior for this route.
    pub fanout: DelegationFanout,
}

impl StaticRouteRule {
    /// Creates a route rule with target ids.
    pub fn new(target_ids: Vec<String>) -> GResult<Self> {
        if target_ids.is_empty() {
            return Err(invalid("static route must define at least one target id"));
        }
        Ok(Self {
            step_kind: None,
            output_schema: None,
            tags: Vec::new(),
            goal_patterns: Vec::new(),
            target_ids,
            fanout: DelegationFanout::Single,
        })
    }
}

/// Static router configuration.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StaticRouterConfig {
    /// Ordered routing rules.
    pub routes: Vec<StaticRouteRule>,
}

/// Deterministic static-router delegation backend.
pub struct StaticRouterDelegationProvider {
    config: StaticRouterConfig,
}

impl StaticRouterDelegationProvider {
    /// Creates a static router provider.
    pub fn new(config: StaticRouterConfig) -> Self {
        Self { config }
    }
}

impl DelegationProvider for StaticRouterDelegationProvider {
    fn delegate(
        &self,
        _tenant: &TenantCtx,
        request: DelegationRequest,
    ) -> GResult<DelegationDecision> {
        if !request.control_allowed {
            return Err(invalid("delegation denied by control policy"));
        }

        let available_by_id: std::collections::BTreeMap<_, _> = request
            .available_targets
            .iter()
            .filter(|target| target.available)
            .map(|target| (target.target_id.clone(), target.clone()))
            .collect();

        for route in &self.config.routes {
            if !matches_route(route, &request) {
                continue;
            }

            let mut targets = Vec::new();
            for target_id in &route.target_ids {
                let Some(target) = available_by_id.get(target_id) else {
                    return Err(invalid(format!("target `{target_id}` is unavailable")));
                };
                targets.push(target.clone());
            }

            let fanout = route.fanout;
            if fanout == DelegationFanout::Single && targets.len() != 1 {
                return Err(invalid(
                    "single fanout routes must resolve to exactly one target",
                ));
            }

            return Ok(DelegationDecision::empty()
                .with_fanout(fanout)
                .with_rationale("static.route", "matched deterministic static route")
                .with_rationale(
                    "static.targets",
                    format!(
                        "selected targets: {}",
                        targets
                            .iter()
                            .map(|target| target.target_id.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )
                .with_targets(targets));
        }

        Ok(DelegationDecision::empty()
            .with_fanout(DelegationFanout::None)
            .with_rationale("static.no_match", "no static route matched"))
    }
}

trait DecisionTargetsExt {
    fn with_targets(self, targets: Vec<DelegationTarget>) -> Self;
}

impl DecisionTargetsExt for DelegationDecision {
    fn with_targets(mut self, targets: Vec<DelegationTarget>) -> Self {
        self.targets.extend(targets);
        self
    }
}

fn matches_route(route: &StaticRouteRule, request: &DelegationRequest) -> bool {
    if let Some(step_kind) = &route.step_kind
        && step_kind != &request.step_kind
    {
        return false;
    }
    if let Some(output_schema) = &route.output_schema
        && request.output_schema.as_ref() != Some(output_schema)
    {
        return false;
    }
    if !route
        .tags
        .iter()
        .all(|tag| request.tags.iter().any(|candidate| candidate == tag))
    {
        return false;
    }
    if !route.goal_patterns.is_empty() {
        let Some(goal) = &request.goal else {
            return false;
        };
        if !route
            .goal_patterns
            .iter()
            .any(|pattern| goal.contains(pattern))
        {
            return false;
        }
    }
    true
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}
