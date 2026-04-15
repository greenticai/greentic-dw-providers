#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Shared planning contract and typed plan document models.

use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Request passed to a planning provider.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanRequest {
    /// Stable request identifier.
    pub request_id: String,
    /// Human-readable planning goal.
    pub goal: String,
    /// Optional machine-readable planning context.
    pub context: Value,
}

impl PlanRequest {
    /// Creates a new plan request.
    pub fn new(request_id: impl Into<String>, goal: impl Into<String>) -> GResult<Self> {
        let request_id = request_id.into();
        let goal = goal.into();
        if request_id.trim().is_empty() {
            return Err(invalid("plan request id must not be empty"));
        }
        if goal.trim().is_empty() {
            return Err(invalid("plan goal must not be empty"));
        }

        Ok(Self {
            request_id,
            goal,
            context: Value::Object(Map::new()),
        })
    }

    /// Attaches optional planning context.
    #[must_use]
    pub fn with_context(mut self, context: Value) -> Self {
        self.context = context;
        self
    }
}

/// A single typed step in a plan document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanStep {
    /// Stable step identifier.
    pub id: String,
    /// Stable step kind used by later routing and execution.
    pub kind: String,
    /// Upstream step identifiers this step depends on.
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Arbitrary step payload.
    #[serde(default = "default_object_value")]
    pub payload: Value,
}

impl PlanStep {
    /// Creates a new plan step.
    pub fn new(id: impl Into<String>, kind: impl Into<String>) -> GResult<Self> {
        let id = id.into();
        let kind = kind.into();
        if id.trim().is_empty() {
            return Err(invalid("plan step id must not be empty"));
        }
        if kind.trim().is_empty() {
            return Err(invalid("plan step kind must not be empty"));
        }

        Ok(Self {
            id,
            kind,
            depends_on: Vec::new(),
            payload: Value::Object(Map::new()),
        })
    }

    /// Declares a dependency on another step.
    #[must_use]
    pub fn with_dependency(mut self, dependency: impl Into<String>) -> Self {
        self.depends_on.push(dependency.into());
        self
    }

    /// Attaches a structured payload to the step.
    #[must_use]
    pub fn with_payload(mut self, payload: Value) -> Self {
        self.payload = payload;
        self
    }
}

/// A normalized plan document returned by a planning provider.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanDocument {
    /// Stable plan identifier.
    pub plan_id: String,
    /// Ordered plan steps.
    pub steps: Vec<PlanStep>,
    /// Optional provider metadata.
    #[serde(default = "default_object_value")]
    pub metadata: Value,
}

impl PlanDocument {
    /// Creates an empty plan document.
    pub fn new(plan_id: impl Into<String>) -> GResult<Self> {
        let plan_id = plan_id.into();
        if plan_id.trim().is_empty() {
            return Err(invalid("plan id must not be empty"));
        }

        Ok(Self {
            plan_id,
            steps: Vec::new(),
            metadata: Value::Object(Map::new()),
        })
    }

    /// Appends a plan step.
    #[must_use]
    pub fn with_step(mut self, step: PlanStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Validates the plan shape for deterministic providers.
    pub fn validate(&self) -> GResult<()> {
        if self.steps.is_empty() {
            return Err(invalid("plan document must contain at least one step"));
        }

        let mut seen = std::collections::BTreeSet::new();
        for step in &self.steps {
            if step.id.trim().is_empty() {
                return Err(invalid("plan step id must not be empty"));
            }
            if step.kind.trim().is_empty() {
                return Err(invalid("plan step kind must not be empty"));
            }
            if !seen.insert(step.id.clone()) {
                return Err(invalid(format!("duplicate plan step id `{}`", step.id)));
            }
        }

        for step in &self.steps {
            for dependency in &step.depends_on {
                if !seen.contains(dependency) {
                    return Err(invalid(format!(
                        "plan step `{}` depends on unknown step `{dependency}`",
                        step.id
                    )));
                }
                if dependency == &step.id {
                    return Err(invalid(format!(
                        "plan step `{}` must not depend on itself",
                        step.id
                    )));
                }
            }
        }

        Ok(())
    }
}

/// Contract implemented by planning backends.
pub trait PlanningProvider: Send + Sync {
    /// Builds a plan document for a request.
    fn plan(&self, tenant: &TenantCtx, request: PlanRequest) -> GResult<PlanDocument>;
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}

fn default_object_value() -> Value {
    Value::Object(Map::new())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{PlanDocument, PlanRequest, PlanStep};
    use serde_json::json;

    #[test]
    fn request_requires_id_and_goal() {
        assert!(PlanRequest::new("", "goal").is_err());
        assert!(PlanRequest::new("req-1", "").is_err());
    }

    #[test]
    fn plan_document_collects_steps() {
        let plan = PlanDocument::new("plan-1").expect("plan").with_step(
            PlanStep::new("step-1", "outline")
                .expect("step")
                .with_payload(json!({"title": "Draft outline"})),
        );

        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].kind, "outline");
        plan.validate().expect("valid plan");
    }

    #[test]
    fn plan_document_rejects_unknown_dependencies() {
        let plan = PlanDocument::new("plan-1").expect("plan").with_step(
            PlanStep::new("step-1", "outline")
                .expect("step")
                .with_dependency("missing"),
        );

        let err = plan.validate().expect_err("unknown dependency");
        assert!(err.to_string().contains("unknown step"));
    }
}
