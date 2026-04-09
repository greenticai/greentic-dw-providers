#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Shared engine contract and decision helpers.

use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Input to an engine decision or routing pass.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EngineRequest {
    /// Stable request identifier.
    pub request_id: String,
    /// Goal or task the engine should evaluate.
    pub goal: String,
    /// Candidate routes or providers the engine may choose from.
    pub candidates: Vec<String>,
    /// Optional free-form context payload.
    pub context: Value,
    /// Optional priority hint.
    pub priority: Option<u32>,
}

impl EngineRequest {
    /// Creates a new engine request.
    pub fn new(request_id: impl Into<String>, goal: impl Into<String>) -> GResult<Self> {
        let request_id = request_id.into();
        let goal = goal.into();
        if request_id.trim().is_empty() {
            return Err(invalid("engine request id must not be empty"));
        }
        if goal.trim().is_empty() {
            return Err(invalid("engine goal must not be empty"));
        }

        Ok(Self {
            request_id,
            goal,
            candidates: Vec::new(),
            context: Value::Null,
            priority: None,
        })
    }

    /// Adds a candidate route or provider.
    #[must_use]
    pub fn with_candidate(mut self, candidate: impl Into<String>) -> Self {
        self.candidates.push(candidate.into());
        self
    }

    /// Sets the context payload.
    #[must_use]
    pub fn with_context(mut self, context: Value) -> Self {
        self.context = context;
        self
    }

    /// Sets the priority hint.
    #[must_use]
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = Some(priority);
        self
    }
}

/// Output returned by an engine provider.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EngineDecision {
    /// Decision type emitted by the engine.
    pub decision_type: String,
    /// Selected route or action identifier.
    pub selected: String,
    /// Human-readable explanation for the choice.
    pub reason: String,
    /// Optional execution plan steps.
    pub plan: Vec<String>,
    /// Free-form details payload.
    pub details: Value,
}

/// Contract implemented by engine providers.
pub trait Engine: Send + Sync {
    /// Produces a default decision for a request.
    fn decide(&self, tenant: &TenantCtx, request: EngineRequest) -> GResult<EngineDecision>;

    /// Routes a request to one of its candidates or a derived route.
    fn route(&self, tenant: &TenantCtx, request: EngineRequest) -> GResult<EngineDecision>;
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::EngineRequest;

    #[test]
    fn request_rejects_empty_goal() {
        let err = match EngineRequest::new("req-1", "   ") {
            Ok(_) => panic!("goal should be invalid"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("goal"));
    }
}
