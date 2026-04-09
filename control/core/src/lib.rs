#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Shared control contract and policy request models.

use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Input evaluated by control providers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlRequest {
    /// Stable request identifier.
    pub request_id: String,
    /// Requested action or operation.
    pub action: String,
    /// Actor requesting the action.
    pub actor: String,
    /// Whether the request attempts delegation.
    pub delegation_requested: bool,
    /// Free-form attributes used for policy matching.
    pub attributes: BTreeMap<String, String>,
    /// Optional context payload.
    pub context: Value,
}

impl ControlRequest {
    /// Creates a new control request.
    pub fn new(
        request_id: impl Into<String>,
        action: impl Into<String>,
        actor: impl Into<String>,
    ) -> GResult<Self> {
        let request_id = request_id.into();
        let action = action.into();
        let actor = actor.into();
        if request_id.trim().is_empty() {
            return Err(invalid("control request id must not be empty"));
        }
        if action.trim().is_empty() {
            return Err(invalid("control action must not be empty"));
        }
        if actor.trim().is_empty() {
            return Err(invalid("control actor must not be empty"));
        }

        Ok(Self {
            request_id,
            action,
            actor,
            delegation_requested: false,
            attributes: BTreeMap::new(),
            context: Value::Null,
        })
    }

    /// Marks the request as a delegation attempt.
    #[must_use]
    pub fn with_delegation_requested(mut self, delegation_requested: bool) -> Self {
        self.delegation_requested = delegation_requested;
        self
    }

    /// Adds an attribute used by policy checks.
    #[must_use]
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Sets a free-form context payload.
    #[must_use]
    pub fn with_context(mut self, context: Value) -> Self {
        self.context = context;
        self
    }
}

/// Decision returned by a control provider.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlDecision {
    /// Whether the request is allowed.
    pub allowed: bool,
    /// Stable policy name or guard name that handled the decision.
    pub policy: String,
    /// Human-readable reason.
    pub reason: String,
    /// Optional normalized details payload.
    pub details: Value,
}

/// Contract implemented by control providers.
pub trait Control: Send + Sync {
    /// Evaluates a request under the provider's policy rules.
    fn evaluate(&self, tenant: &TenantCtx, request: ControlRequest) -> GResult<ControlDecision>;

    /// Applies guard-specific delegation checks.
    fn guard(&self, tenant: &TenantCtx, request: ControlRequest) -> GResult<ControlDecision>;
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::ControlRequest;

    #[test]
    fn control_request_rejects_empty_actor() {
        let err = match ControlRequest::new("req-1", "action", "   ") {
            Ok(_) => panic!("actor should be invalid"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("actor"));
    }
}
