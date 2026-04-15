#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Shared delegation routing contract and decision models.

use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};
use serde::{Deserialize, Serialize};
use serde_json::Value;
/// Requested delegation fanout mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationFanout {
    /// Do not select any targets.
    None,
    /// Select exactly one target.
    #[default]
    Single,
    /// Select multiple targets in deterministic order.
    Parallel,
}

/// A candidate delegation target.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationTarget {
    /// Stable target identifier.
    pub target_id: String,
    /// Optional declared capability set for the target.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Optional supported output schema identifiers.
    #[serde(default)]
    pub schemas: Vec<String>,
    /// Optional routing tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Whether the target is currently routable.
    #[serde(default = "default_true")]
    pub available: bool,
}

impl DelegationTarget {
    /// Creates a validated target id.
    pub fn new(target_id: impl Into<String>) -> GResult<Self> {
        let target_id = target_id.into();
        if target_id.trim().is_empty() {
            return Err(invalid("delegation target id must not be empty"));
        }
        Ok(Self {
            target_id,
            capabilities: Vec::new(),
            schemas: Vec::new(),
            tags: Vec::new(),
            available: true,
        })
    }
}

/// A routing request for a plan step or subtask.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DelegationRequest {
    /// Stable request identifier.
    pub request_id: String,
    /// Step kind to route.
    pub step_kind: String,
    /// Optional goal text associated with the step.
    pub goal: Option<String>,
    /// Optional schema identifier expected from the target.
    pub output_schema: Option<String>,
    /// Optional routing tags.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Optional structured requirements.
    pub requirements: Value,
    /// Optional expected capabilities.
    #[serde(default)]
    pub required_capabilities: Vec<String>,
    /// Targets currently visible to the router.
    #[serde(default)]
    pub available_targets: Vec<DelegationTarget>,
    /// Requested fanout behavior.
    #[serde(default)]
    pub fanout: DelegationFanout,
    /// Whether control policy still permits delegation.
    #[serde(default = "default_true")]
    pub control_allowed: bool,
}

impl DelegationRequest {
    /// Creates a new routing request.
    pub fn new(request_id: impl Into<String>, step_kind: impl Into<String>) -> GResult<Self> {
        let request_id = request_id.into();
        let step_kind = step_kind.into();
        if request_id.trim().is_empty() {
            return Err(invalid("delegation request id must not be empty"));
        }
        if step_kind.trim().is_empty() {
            return Err(invalid("delegation step kind must not be empty"));
        }

        Ok(Self {
            request_id,
            step_kind,
            goal: None,
            output_schema: None,
            tags: Vec::new(),
            requirements: Value::Null,
            required_capabilities: Vec::new(),
            available_targets: Vec::new(),
            fanout: DelegationFanout::Single,
            control_allowed: true,
        })
    }

    /// Sets the goal text for routing.
    #[must_use]
    pub fn with_goal(mut self, goal: impl Into<String>) -> Self {
        self.goal = Some(goal.into());
        self
    }

    /// Sets the expected output schema id.
    #[must_use]
    pub fn with_output_schema(mut self, output_schema: impl Into<String>) -> Self {
        self.output_schema = Some(output_schema.into());
        self
    }

    /// Appends a routing tag.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Appends a required capability.
    #[must_use]
    pub fn with_required_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capabilities.push(capability.into());
        self
    }

    /// Adds a visible target to the routing request.
    #[must_use]
    pub fn with_available_target(mut self, target: DelegationTarget) -> Self {
        self.available_targets.push(target);
        self
    }

    /// Sets the fanout mode.
    #[must_use]
    pub fn with_fanout(mut self, fanout: DelegationFanout) -> Self {
        self.fanout = fanout;
        self
    }

    /// Marks whether delegation remains allowed by control policy.
    #[must_use]
    pub fn with_control_allowed(mut self, control_allowed: bool) -> Self {
        self.control_allowed = control_allowed;
        self
    }
}

/// A human-readable rationale item for an outcome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationRationale {
    /// Stable explanation code.
    pub code: String,
    /// Human-readable explanation text.
    pub message: String,
}

/// Result produced by a delegation provider.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DelegationDecision {
    /// Explicitly selected targets.
    pub targets: Vec<DelegationTarget>,
    /// Final fanout mode used for the decision.
    pub fanout: DelegationFanout,
    /// Supporting rationale entries.
    pub rationale: Vec<DelegationRationale>,
}

impl DelegationDecision {
    /// Creates an empty routing decision.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            targets: Vec::new(),
            fanout: DelegationFanout::None,
            rationale: Vec::new(),
        }
    }

    /// Adds a selected target.
    #[must_use]
    pub fn with_target(mut self, target: DelegationTarget) -> Self {
        self.targets.push(target);
        self
    }

    /// Sets the fanout mode on the decision.
    #[must_use]
    pub fn with_fanout(mut self, fanout: DelegationFanout) -> Self {
        self.fanout = fanout;
        self
    }

    /// Appends a rationale item.
    #[must_use]
    pub fn with_rationale(mut self, code: impl Into<String>, message: impl Into<String>) -> Self {
        self.rationale.push(DelegationRationale {
            code: code.into(),
            message: message.into(),
        });
        self
    }
}

/// Contract implemented by delegation backends.
pub trait DelegationProvider: Send + Sync {
    /// Selects delegation targets for a request.
    fn delegate(
        &self,
        tenant: &TenantCtx,
        request: DelegationRequest,
    ) -> GResult<DelegationDecision>;
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{DelegationDecision, DelegationFanout, DelegationRequest, DelegationTarget};

    #[test]
    fn request_requires_fields() {
        assert!(DelegationRequest::new("", "plan").is_err());
        assert!(DelegationRequest::new("req-1", "").is_err());
    }

    #[test]
    fn decision_tracks_targets() {
        let decision = DelegationDecision::empty()
            .with_fanout(DelegationFanout::Single)
            .with_target(DelegationTarget::new("agent-a").expect("target"));
        assert_eq!(decision.targets.len(), 1);
        assert_eq!(decision.targets[0].target_id, "agent-a");
        assert_eq!(decision.fanout, DelegationFanout::Single);
    }
}
