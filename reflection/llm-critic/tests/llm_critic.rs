use greentic_dw_reflection::{ReflectionProvider, ReviewDisposition, ReviewRequest};
use greentic_dw_reflection_llm_critic::{
    CriticModel, LlmCriticConfig, LlmCriticReflectionProvider,
};
use greentic_types::{EnvId, GResult, TenantCtx, TenantId};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

fn tenant() -> TenantCtx {
    TenantCtx::new(
        EnvId::try_from("dev").expect("env id"),
        TenantId::try_from("tenant-reflection").expect("tenant id"),
    )
}

#[derive(Clone)]
struct StubCritic {
    responses: Arc<Mutex<VecDeque<String>>>,
}

impl StubCritic {
    fn new(responses: Vec<&str>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(
                responses.into_iter().map(ToString::to_string).collect(),
            )),
        }
    }
}

impl CriticModel for StubCritic {
    fn generate_review_json(&self, _prompt: &str) -> GResult<String> {
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
fn llm_critic_output_is_validated_strictly() {
    let provider = LlmCriticReflectionProvider::new(
        LlmCriticConfig::new("dw.llm.openai"),
        StubCritic::new(vec![
            r#"{"disposition":"Revise","findings":[{"code":"","message":"missing"}]}"#,
            r#"{"disposition":"Revise","findings":[{"code":"critic.issue","message":"missing title"}]}"#,
        ]),
    )
    .expect("provider");

    let outcome = provider
        .review(
            &tenant(),
            ReviewRequest::new("req-1", serde_json::json!({"title": null})).expect("request"),
        )
        .expect("outcome");

    assert_eq!(outcome.disposition, ReviewDisposition::Revise);
    assert_eq!(outcome.findings.len(), 1);
    assert_eq!(outcome.findings[0].code, "critic.issue");
}
