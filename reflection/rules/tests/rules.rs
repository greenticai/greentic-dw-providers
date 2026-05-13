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

#[test]
fn rules_provider_accepts_when_all_rules_pass() {
    let provider = RulesReflectionProvider::new(vec![
        Rule::Exists {
            path: "$.title".to_string(),
            code: "rules.exists".to_string(),
            message: "title is required".to_string(),
        },
        Rule::ScoreGte {
            path: "$.score".to_string(),
            min: 0.5,
            code: "rules.score".to_string(),
            message: "score must be high enough".to_string(),
        },
    ]);

    let outcome = provider
        .review(
            &tenant(),
            ReviewRequest::new("req-2", json!({"title": "ok", "score": 0.9})).expect("request"),
        )
        .expect("outcome");

    assert_eq!(outcome.disposition, ReviewDisposition::Accept);
    assert!(outcome.findings.is_empty());
}

#[test]
fn rules_provider_accepts_when_no_rules_configured() {
    let provider = RulesReflectionProvider::new(Vec::new());

    let outcome = provider
        .review(
            &tenant(),
            ReviewRequest::new("req-empty", json!({"anything": true})).expect("request"),
        )
        .expect("outcome");

    assert_eq!(outcome.disposition, ReviewDisposition::Accept);
    assert!(outcome.findings.is_empty());
}

#[test]
fn contains_rule_passes_when_substring_present_and_fails_when_missing() {
    let rule = Rule::Contains {
        path: "$.summary".to_string(),
        needle: "deterministic".to_string(),
        code: "rules.contains".to_string(),
        message: "summary must mention deterministic".to_string(),
    };

    let pass = RulesReflectionProvider::new(vec![rule.clone()])
        .review(
            &tenant(),
            ReviewRequest::new("req-c1", json!({"summary": "fully deterministic flow"}))
                .expect("request"),
        )
        .expect("outcome");
    assert_eq!(pass.disposition, ReviewDisposition::Accept);

    let fail = RulesReflectionProvider::new(vec![rule.clone()])
        .review(
            &tenant(),
            ReviewRequest::new("req-c2", json!({"summary": "something else"})).expect("request"),
        )
        .expect("outcome");
    assert_eq!(fail.disposition, ReviewDisposition::Revise);
    assert_eq!(fail.findings.len(), 1);
    assert_eq!(fail.findings[0].code, "rules.contains");

    let missing = RulesReflectionProvider::new(vec![rule])
        .review(
            &tenant(),
            ReviewRequest::new("req-c3", json!({})).expect("request"),
        )
        .expect("outcome");
    assert_eq!(missing.disposition, ReviewDisposition::Revise);
}

#[test]
fn contains_rule_stringifies_non_string_values() {
    let rule = Rule::Contains {
        path: "$.value".to_string(),
        needle: "42".to_string(),
        code: "rules.contains.num".to_string(),
        message: "value must contain 42".to_string(),
    };

    let outcome = RulesReflectionProvider::new(vec![rule])
        .review(
            &tenant(),
            ReviewRequest::new("req-c-num", json!({"value": 1042})).expect("request"),
        )
        .expect("outcome");
    assert_eq!(outcome.disposition, ReviewDisposition::Accept);
}

#[test]
fn count_gte_handles_arrays_objects_and_strings() {
    let rule = |path: &str, min: usize| Rule::CountGte {
        path: path.to_string(),
        min,
        code: "rules.count".to_string(),
        message: "count too low".to_string(),
    };

    let provider = RulesReflectionProvider::new(vec![
        rule("$.items", 2),
        rule("$.bag", 2),
        rule("$.label", 3),
    ]);

    let pass = provider
        .review(
            &tenant(),
            ReviewRequest::new(
                "req-cg-1",
                json!({
                    "items": [1, 2, 3],
                    "bag": {"a": 1, "b": 2},
                    "label": "abcd"
                }),
            )
            .expect("request"),
        )
        .expect("outcome");
    assert_eq!(pass.disposition, ReviewDisposition::Accept);

    let fail = provider
        .review(
            &tenant(),
            ReviewRequest::new(
                "req-cg-2",
                json!({"items": [1], "bag": {"a": 1}, "label": "ab"}),
            )
            .expect("request"),
        )
        .expect("outcome");
    assert_eq!(fail.disposition, ReviewDisposition::Revise);
    assert_eq!(fail.findings.len(), 3);
}

#[test]
fn count_gte_treats_missing_or_unsupported_value_as_zero() {
    let provider = RulesReflectionProvider::new(vec![
        Rule::CountGte {
            path: "$.missing".to_string(),
            min: 1,
            code: "rules.count.missing".to_string(),
            message: "missing".to_string(),
        },
        Rule::CountGte {
            path: "$.flag".to_string(),
            min: 1,
            code: "rules.count.bool".to_string(),
            message: "bool counts as 0".to_string(),
        },
    ]);

    let outcome = provider
        .review(
            &tenant(),
            ReviewRequest::new("req-cg-3", json!({"flag": true})).expect("request"),
        )
        .expect("outcome");

    assert_eq!(outcome.disposition, ReviewDisposition::Revise);
    assert_eq!(outcome.findings.len(), 2);
}

#[test]
fn score_gte_treats_non_numeric_values_as_failure() {
    let provider = RulesReflectionProvider::new(vec![Rule::ScoreGte {
        path: "$.score".to_string(),
        min: 0.0,
        code: "rules.score.nan".to_string(),
        message: "score must be numeric".to_string(),
    }]);

    let outcome = provider
        .review(
            &tenant(),
            ReviewRequest::new("req-s1", json!({"score": "not-a-number"})).expect("request"),
        )
        .expect("outcome");

    assert_eq!(outcome.disposition, ReviewDisposition::Revise);
    assert_eq!(outcome.findings.len(), 1);
}

#[test]
fn schema_valid_rejects_when_path_missing() {
    let provider = RulesReflectionProvider::new(vec![Rule::SchemaValid {
        path: "$.body".to_string(),
        schema: json!({"type": "object"}),
        code: "rules.schema.missing".to_string(),
        message: "body required".to_string(),
    }]);

    let outcome = provider
        .review(
            &tenant(),
            ReviewRequest::new("req-sv-1", json!({})).expect("request"),
        )
        .expect("outcome");

    assert_eq!(outcome.disposition, ReviewDisposition::Revise);
    assert_eq!(outcome.findings[0].code, "rules.schema.missing");
}

#[test]
fn schema_valid_validates_each_supported_type() {
    let cases: Vec<(&str, serde_json::Value, serde_json::Value)> = vec![
        ("object", json!({"type": "object"}), json!({"k": 1})),
        ("array", json!({"type": "array"}), json!([1, 2])),
        ("string", json!({"type": "string"}), json!("hi")),
        ("number", json!({"type": "number"}), json!(1.5)),
        ("integer", json!({"type": "integer"}), json!(7)),
        ("integer-u64", json!({"type": "integer"}), json!(u64::MAX)),
        ("boolean", json!({"type": "boolean"}), json!(true)),
        ("null", json!({"type": "null"}), json!(null)),
        ("unknown", json!({"type": "anything-goes"}), json!("x")),
        ("no-type", json!({}), json!("anything")),
    ];

    for (label, schema, value) in cases {
        let provider = RulesReflectionProvider::new(vec![Rule::SchemaValid {
            path: "$.field".to_string(),
            schema,
            code: "rules.schema.ok".to_string(),
            message: format!("{label} should pass"),
        }]);

        let outcome = provider
            .review(
                &tenant(),
                ReviewRequest::new("req-sv-ok", json!({"field": value})).expect("request"),
            )
            .expect("outcome");
        assert_eq!(
            outcome.disposition,
            ReviewDisposition::Accept,
            "expected accept for {label}"
        );
    }
}

#[test]
fn schema_valid_rejects_each_supported_type_mismatch() {
    let cases: Vec<(&str, serde_json::Value, serde_json::Value)> = vec![
        ("object", json!({"type": "object"}), json!("not-object")),
        ("array", json!({"type": "array"}), json!("not-array")),
        ("string", json!({"type": "string"}), json!(1)),
        ("number", json!({"type": "number"}), json!("not-number")),
        ("integer", json!({"type": "integer"}), json!(1.5)),
        ("boolean", json!({"type": "boolean"}), json!("not-bool")),
        ("null", json!({"type": "null"}), json!("not-null")),
    ];

    for (label, schema, value) in cases {
        let provider = RulesReflectionProvider::new(vec![Rule::SchemaValid {
            path: "$.field".to_string(),
            schema,
            code: "rules.schema.bad".to_string(),
            message: format!("{label} should fail"),
        }]);

        let outcome = provider
            .review(
                &tenant(),
                ReviewRequest::new("req-sv-bad", json!({"field": value})).expect("request"),
            )
            .expect("outcome");
        assert_eq!(
            outcome.disposition,
            ReviewDisposition::Revise,
            "expected revise for {label}"
        );
    }
}

#[test]
fn root_path_resolves_to_subject() {
    let provider = RulesReflectionProvider::new(vec![Rule::SchemaValid {
        path: "$".to_string(),
        schema: json!({"type": "object"}),
        code: "rules.root".to_string(),
        message: "root must be object".to_string(),
    }]);

    let outcome = provider
        .review(
            &tenant(),
            ReviewRequest::new("req-root", json!({"any": 1})).expect("request"),
        )
        .expect("outcome");
    assert_eq!(outcome.disposition, ReviewDisposition::Accept);
}

#[test]
fn empty_path_resolves_to_subject() {
    let provider = RulesReflectionProvider::new(vec![Rule::Exists {
        path: String::new(),
        code: "rules.exists.root".to_string(),
        message: "root must exist".to_string(),
    }]);

    let outcome = provider
        .review(
            &tenant(),
            ReviewRequest::new("req-empty-path", json!(null)).expect("request"),
        )
        .expect("outcome");
    assert_eq!(outcome.disposition, ReviewDisposition::Accept);
}

#[test]
fn nested_path_traverses_objects() {
    let provider = RulesReflectionProvider::new(vec![Rule::Exists {
        path: "$.outer.inner".to_string(),
        code: "rules.nested".to_string(),
        message: "nested.inner must exist".to_string(),
    }]);

    let pass = provider
        .review(
            &tenant(),
            ReviewRequest::new("req-nested-1", json!({"outer": {"inner": 1}})).expect("request"),
        )
        .expect("outcome");
    assert_eq!(pass.disposition, ReviewDisposition::Accept);

    let fail = provider
        .review(
            &tenant(),
            ReviewRequest::new("req-nested-2", json!({"outer": {"other": 1}})).expect("request"),
        )
        .expect("outcome");
    assert_eq!(fail.disposition, ReviewDisposition::Revise);

    let non_object = provider
        .review(
            &tenant(),
            ReviewRequest::new("req-nested-3", json!({"outer": 1})).expect("request"),
        )
        .expect("outcome");
    assert_eq!(non_object.disposition, ReviewDisposition::Revise);
}
