use crate::config::{LlmOutlineConfig, ReplanPromptVariant};
use greentic_dw_planning::PlanRequest;

pub(crate) fn build_outline_prompt(config: &LlmOutlineConfig, request: &PlanRequest) -> String {
    let allowed = if config.allowed_step_kinds.is_empty() {
        "any non-empty planning step kinds".to_string()
    } else {
        config.allowed_step_kinds.join(", ")
    };

    format!(
        "You are a planning backend. Return only JSON for a PlanDocument with fields \
         `plan_id`, `steps`, and optional `metadata`. Each step must contain `id`, `kind`, \
         optional `depends_on`, and optional `payload`. Goal: {}. Request id: {}. \
         Maximum steps: {}. Allowed step kinds: {}. Do not include prose.",
        request.goal, request.request_id, config.max_steps, allowed
    )
}

pub(crate) fn build_replan_prompt(
    config: &LlmOutlineConfig,
    request: &PlanRequest,
    feedback: &str,
) -> String {
    let base = build_outline_prompt(config, request);
    match config.replan_prompt_variant {
        ReplanPromptVariant::ValidatorFeedback => {
            format!("{base} The previous plan was invalid: {feedback}. Return corrected JSON only.")
        }
        ReplanPromptVariant::JsonRepair => {
            format!("{base} Repair the response and return valid JSON only.")
        }
    }
}
