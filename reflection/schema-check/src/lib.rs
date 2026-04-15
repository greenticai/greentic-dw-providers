#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Deterministic schema validation reflection provider.

use greentic_dw_reflection::{
    ReflectionProvider, ReviewDisposition, ReviewFinding, ReviewOutcome, ReviewRequest,
};
use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};
use serde_json::Value;

/// Reflection provider that checks a subject against a supplied schema.
#[derive(Default)]
pub struct SchemaCheckReflectionProvider;

impl SchemaCheckReflectionProvider {
    /// Creates a new schema-check provider.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl ReflectionProvider for SchemaCheckReflectionProvider {
    fn review(&self, _tenant: &TenantCtx, request: ReviewRequest) -> GResult<ReviewOutcome> {
        let Some(schema) = request.schema.as_ref() else {
            return Err(invalid("schema-check requires review_request.schema"));
        };
        let mut findings = Vec::new();
        validate_against_schema(&request.subject, schema, "$", &mut findings);
        if findings.is_empty() {
            Ok(ReviewOutcome::accept())
        } else {
            Ok(ReviewOutcome {
                disposition: ReviewDisposition::Revise,
                findings,
            })
        }
    }
}

fn validate_against_schema(
    subject: &Value,
    schema: &Value,
    path: &str,
    findings: &mut Vec<ReviewFinding>,
) {
    if let Some(expected_type) = schema.get("type").and_then(Value::as_str)
        && !matches_type(subject, expected_type)
    {
        findings.push(ReviewFinding {
            code: "schema.type".to_string(),
            message: format!("{path} expected type `{expected_type}`"),
        });
        return;
    }

    if let Some(required) = schema.get("required").and_then(Value::as_array)
        && let Some(object) = subject.as_object()
    {
        for key in required.iter().filter_map(Value::as_str) {
            if !object.contains_key(key) {
                findings.push(ReviewFinding {
                    code: "schema.required".to_string(),
                    message: format!("{path} missing required field `{key}`"),
                });
            }
        }
    }

    if let (Some(properties), Some(object)) = (schema.get("properties"), subject.as_object())
        && let Some(properties) = properties.as_object()
    {
        for (key, property_schema) in properties {
            if let Some(value) = object.get(key) {
                let child_path = format!("{path}.{key}");
                validate_against_schema(value, property_schema, &child_path, findings);
            }
        }
    }

    if let (Some(items_schema), Some(array)) = (schema.get("items"), subject.as_array()) {
        for (index, item) in array.iter().enumerate() {
            let child_path = format!("{path}[{index}]");
            validate_against_schema(item, items_schema, &child_path, findings);
        }
    }
}

fn matches_type(value: &Value, expected_type: &str) -> bool {
    match expected_type {
        "object" => value.is_object(),
        "array" => value.is_array(),
        "string" => value.is_string(),
        "number" => value.is_number(),
        "integer" => value.as_i64().is_some() || value.as_u64().is_some(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => true,
    }
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}
