#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Capability-based delegation router.

use greentic_dw_delegation::{
    DelegationDecision, DelegationFanout, DelegationProvider, DelegationRequest, DelegationTarget,
};
use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};

/// Delegation router that matches visible targets against declared capabilities.
#[derive(Default)]
pub struct CapabilityMatchDelegationProvider;

impl CapabilityMatchDelegationProvider {
    /// Creates a new capability-match provider.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl DelegationProvider for CapabilityMatchDelegationProvider {
    fn delegate(
        &self,
        _tenant: &TenantCtx,
        request: DelegationRequest,
    ) -> GResult<DelegationDecision> {
        if !request.control_allowed {
            return Err(invalid("delegation denied by control policy"));
        }

        let mut matches: Vec<DelegationTarget> = request
            .available_targets
            .iter()
            .filter(|target| target.available)
            .filter(|target| matches_capabilities(target, &request))
            .cloned()
            .collect();
        matches.sort_by(|left, right| left.target_id.cmp(&right.target_id));

        if request.fanout == DelegationFanout::None {
            return Ok(DelegationDecision::empty()
                .with_fanout(DelegationFanout::None)
                .with_rationale("capability.none", "fanout requested no delegation"));
        }

        if matches.is_empty() {
            return Err(invalid(
                "no available agent matched the requested capabilities",
            ));
        }

        let selected = match request.fanout {
            DelegationFanout::Single => vec![matches.remove(0)],
            DelegationFanout::Parallel => matches,
            DelegationFanout::None => Vec::new(),
        };

        Ok(DelegationDecision {
            fanout: request.fanout,
            rationale: vec![
                greentic_dw_delegation::DelegationRationale {
                    code: "capability.match".to_string(),
                    message: "selected targets by declared capabilities".to_string(),
                },
                greentic_dw_delegation::DelegationRationale {
                    code: "capability.targets".to_string(),
                    message: format!(
                        "selected targets: {}",
                        selected
                            .iter()
                            .map(|target| target.target_id.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                },
            ],
            targets: selected,
        })
    }
}

fn matches_capabilities(target: &DelegationTarget, request: &DelegationRequest) -> bool {
    if !request
        .required_capabilities
        .iter()
        .all(|capability| target.capabilities.iter().any(|value| value == capability))
    {
        return false;
    }
    if let Some(schema) = &request.output_schema
        && !target.schemas.is_empty()
        && !target.schemas.iter().any(|value| value == schema)
    {
        return false;
    }
    if !request.tags.is_empty()
        && !request
            .tags
            .iter()
            .all(|tag| target.tags.iter().any(|candidate| candidate == tag))
    {
        return false;
    }
    true
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}
