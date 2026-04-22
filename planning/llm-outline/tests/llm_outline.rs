use greentic_dw_planning::{PlanRequest, PlanningProvider};
use greentic_dw_planning_llm_outline::{
    LlmOutlineConfig, LlmOutlinePlanningProvider, OutlineModel, ReplanPromptVariant,
    SchemaStrictness,
};
use greentic_types::{EnvId, GResult, TenantCtx, TenantId};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

fn tenant() -> TenantCtx {
    TenantCtx::new(
        EnvId::try_from("dev").expect("env id"),
        TenantId::try_from("tenant-planning").expect("tenant id"),
    )
}

#[derive(Clone)]
struct StubModel {
    responses: Arc<Mutex<VecDeque<String>>>,
    prompts: Arc<Mutex<Vec<String>>>,
}

impl StubModel {
    fn new(responses: Vec<&str>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(
                responses.into_iter().map(ToString::to_string).collect(),
            )),
            prompts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn prompt_count(&self) -> usize {
        self.prompts.lock().expect("prompts").len()
    }
}

impl OutlineModel for StubModel {
    fn generate_plan_json(&self, prompt: &str) -> GResult<String> {
        self.prompts
            .lock()
            .expect("prompts")
            .push(prompt.to_string());
        self.responses
            .lock()
            .expect("responses")
            .pop_front()
            .ok_or_else(|| {
                greentic_types::GreenticError::new(
                    greentic_types::ErrorCode::Internal,
                    "missing stub response",
                )
            })
    }
}

#[test]
fn llm_outline_rejects_malformed_graph() {
    let model = StubModel::new(vec![
        r#"{"plan_id":"bad","steps":[{"id":"step-1","kind":"research","depends_on":["missing"],"payload":{}}]}"#,
        r#"{"plan_id":"still-bad","steps":[]}"#,
    ]);
    let mut config = LlmOutlineConfig::new("dw.llm.openai");
    config.allowed_step_kinds = vec!["research".to_string(), "draft".to_string()];
    let provider = LlmOutlinePlanningProvider::new(config, model).expect("provider");

    let err = provider
        .plan(
            &tenant(),
            PlanRequest::new("req-1", "draft an answer").expect("request"),
        )
        .expect_err("invalid graph");

    assert!(
        err.to_string().contains("plan document")
            || err.to_string().contains("unknown step")
            || err.to_string().contains("invalid")
    );
}

#[test]
fn llm_outline_retries_once_with_validator_feedback() {
    let model = StubModel::new(vec![
        r#"{"plan_id":"bad","steps":[{"id":"step-1","kind":"unknown","payload":{}}]}"#,
        r#"{"plan_id":"good","steps":[{"id":"step-1","kind":"research","payload":{}},{"id":"step-2","kind":"draft","depends_on":["step-1"],"payload":{}}]}"#,
    ]);
    let mut config = LlmOutlineConfig::new("dw.llm.openai");
    config.allowed_step_kinds = vec!["research".to_string(), "draft".to_string()];
    config.replan_prompt_variant = ReplanPromptVariant::ValidatorFeedback;
    let provider = LlmOutlinePlanningProvider::new(config, model.clone()).expect("provider");

    let plan = provider
        .plan(
            &tenant(),
            PlanRequest::new("req-2", "draft an answer").expect("request"),
        )
        .expect("plan");

    assert_eq!(plan.plan_id, "good");
    assert_eq!(model.prompt_count(), 2);
}

#[test]
fn llm_outline_preserves_deterministic_validation_behavior() {
    let model = StubModel::new(vec![
        r#"{"plan_id":"good","steps":[{"id":"step-1","kind":"research","payload":{}},{"id":"step-2","kind":"draft","depends_on":["step-1"],"payload":{}}]}"#,
        r#"{"plan_id":"good","steps":[{"id":"step-1","kind":"research","payload":{}},{"id":"step-2","kind":"draft","depends_on":["step-1"],"payload":{}}]}"#,
    ]);
    let mut config = LlmOutlineConfig::new("dw.llm.openai");
    config.max_steps = 2;
    config.allowed_step_kinds = vec!["research".to_string(), "draft".to_string()];
    config.schema_strictness = SchemaStrictness::Strict;
    let provider = LlmOutlinePlanningProvider::new(config, model).expect("provider");

    let first = provider
        .plan(
            &tenant(),
            PlanRequest::new("req-3", "draft an answer").expect("request"),
        )
        .expect("first");
    let second = provider
        .plan(
            &tenant(),
            PlanRequest::new("req-3", "draft an answer").expect("request"),
        )
        .expect("second");

    assert_eq!(first, second);
}
