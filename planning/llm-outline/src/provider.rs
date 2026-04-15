use crate::config::{LlmOutlineConfig, SchemaStrictness};
use crate::prompt::{build_outline_prompt, build_replan_prompt};
use greentic_dw_planning::{PlanDocument, PlanRequest, PlanningProvider};
use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};
use serde_json::Value;

/// Minimal model adapter used by the outline planner.
pub trait OutlineModel: Send + Sync {
    /// Generates a JSON plan string for the supplied prompt.
    fn generate_plan_json(&self, prompt: &str) -> GResult<String>;
}

/// Strict outline planner that validates LLM output as a plan document.
pub struct LlmOutlinePlanningProvider<M> {
    config: LlmOutlineConfig,
    model: M,
}

impl<M> LlmOutlinePlanningProvider<M> {
    /// Creates a new outline planning provider.
    pub fn new(config: LlmOutlineConfig, model: M) -> GResult<Self> {
        if config.llm_provider_ref.trim().is_empty() {
            return Err(invalid("llm provider ref must not be empty"));
        }
        if config.max_steps == 0 {
            return Err(invalid("max steps must be greater than zero"));
        }
        Ok(Self { config, model })
    }
}

impl<M: OutlineModel> PlanningProvider for LlmOutlinePlanningProvider<M> {
    fn plan(&self, _tenant: &TenantCtx, request: PlanRequest) -> GResult<PlanDocument> {
        let first_prompt = build_outline_prompt(&self.config, &request);
        let first = self.model.generate_plan_json(&first_prompt)?;
        match validate_candidate(&self.config, &first) {
            Ok(plan) => Ok(plan),
            Err(err) => {
                let retry_prompt = build_replan_prompt(&self.config, &request, &err.to_string());
                let second = self.model.generate_plan_json(&retry_prompt)?;
                validate_candidate(&self.config, &second)
            }
        }
    }
}

fn validate_candidate(config: &LlmOutlineConfig, candidate: &str) -> GResult<PlanDocument> {
    let value: Value = serde_json::from_str(candidate)
        .map_err(|err| invalid(format!("planner returned invalid json: {err}")))?;
    let plan: PlanDocument = serde_json::from_value(value)
        .map_err(|err| invalid(format!("planner returned invalid plan shape: {err}")))?;
    plan.validate()?;

    if plan.steps.len() > config.max_steps {
        return Err(invalid(format!(
            "planner returned {} steps, exceeding max_steps {}",
            plan.steps.len(),
            config.max_steps
        )));
    }

    if matches!(config.schema_strictness, SchemaStrictness::Strict)
        && !config.allowed_step_kinds.is_empty()
    {
        for step in &plan.steps {
            if !config
                .allowed_step_kinds
                .iter()
                .any(|kind| kind == &step.kind)
            {
                return Err(invalid(format!(
                    "planner returned disallowed step kind `{}`",
                    step.kind
                )));
            }
        }
    }

    Ok(plan)
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}
