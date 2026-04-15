use greentic_dw_reflection::{ReflectionProvider, ReviewDisposition, ReviewRequest};
use greentic_dw_reflection_schema_check::SchemaCheckReflectionProvider;
use greentic_types::{EnvId, TenantCtx, TenantId};
use serde_json::json;

fn tenant() -> TenantCtx {
    TenantCtx::new(
        EnvId::try_from("dev").expect("env id"),
        TenantId::try_from("tenant-reflection").expect("tenant id"),
    )
}

#[test]
fn schema_check_accepts_valid_output_and_rejects_invalid_output() {
    let provider = SchemaCheckReflectionProvider::new();
    let schema = json!({
        "type": "object",
        "required": ["title"],
        "properties": {
            "title": { "type": "string" },
            "score": { "type": "number" }
        }
    });

    let accepted = provider
        .review(
            &tenant(),
            ReviewRequest::new("req-1", json!({"title": "draft", "score": 0.9}))
                .expect("request")
                .with_schema(schema.clone()),
        )
        .expect("accepted");
    assert_eq!(accepted.disposition, ReviewDisposition::Accept);

    let rejected = provider
        .review(
            &tenant(),
            ReviewRequest::new("req-2", json!({"score": "high"}))
                .expect("request")
                .with_schema(schema),
        )
        .expect("rejected");
    assert_eq!(rejected.disposition, ReviewDisposition::Revise);
    assert!(!rejected.findings.is_empty());
}
