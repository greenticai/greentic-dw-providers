use greentic_dw_reflection::{ReflectionProvider, ReviewDisposition, ReviewRequest};
use greentic_dw_reflection_rules::{Rule, RulesReflectionProvider};
use greentic_types::{EnvId, TenantCtx, TenantId};
use serde_json::json;

fn tenant() -> TenantCtx {
    TenantCtx::new(
        EnvId::try_from("dev").expect("env id"),
        TenantId::try_from("tenant-reflection").expect("tenant id"),
    )
}

#[test]
fn rules_provider_returns_revise_with_findings() {
    let provider = RulesReflectionProvider::new(vec![
        Rule::Exists {
            path: "$.title".to_string(),
            code: "rules.exists".to_string(),
            message: "title is required".to_string(),
        },
        Rule::ScoreGte {
            path: "$.score".to_string(),
            min: 0.8,
            code: "rules.score".to_string(),
            message: "score must be high enough".to_string(),
        },
    ]);

    let outcome = provider
        .review(
            &tenant(),
            ReviewRequest::new("req-1", json!({"score": 0.4})).expect("request"),
        )
        .expect("outcome");

    assert_eq!(outcome.disposition, ReviewDisposition::Revise);
    assert_eq!(outcome.findings.len(), 2);
}
