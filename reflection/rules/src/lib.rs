#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Deterministic rule-based reflection provider.

use greentic_dw_reflection::{
    ReflectionProvider, ReviewDisposition, ReviewFinding, ReviewOutcome, ReviewRequest,
};
use greentic_types::{GResult, TenantCtx};
use serde_json::Value;

/// Minimal deterministic rule DSL.
#[derive(Clone, Debug, PartialEq)]
pub enum Rule {
    /// Require a value to exist at a path.
    Exists {
        /// Dot-path into the reviewed subject, for example `$.title`.
        path: String,
        /// Stable machine-readable finding code.
        code: String,
        /// Human-readable finding detail.
        message: String,
    },
    /// Require a stringified value at a path to contain a needle.
    Contains {
        /// Dot-path into the reviewed subject, for example `$.summary`.
        path: String,
        /// Substring that must appear in the value.
        needle: String,
        /// Stable machine-readable finding code.
        code: String,
        /// Human-readable finding detail.
        message: String,
    },
    /// Require an array/object/string count at a path to be at least a minimum.
    CountGte {
        /// Dot-path into the reviewed subject.
        path: String,
        /// Minimum required count.
        min: usize,
        /// Stable machine-readable finding code.
        code: String,
        /// Human-readable finding detail.
        message: String,
    },
    /// Require a numeric score at a path to be at least a minimum.
    ScoreGte {
        /// Dot-path into the reviewed subject.
        path: String,
        /// Minimum acceptable numeric score.
        min: f64,
        /// Stable machine-readable finding code.
        code: String,
        /// Human-readable finding detail.
        message: String,
    },
    /// Require a value at a path to validate against a schema subset.
    SchemaValid {
        /// Dot-path into the reviewed subject.
        path: String,
        /// Schema subset used for validation.
        schema: Value,
        /// Stable machine-readable finding code.
        code: String,
        /// Human-readable finding detail.
        message: String,
    },
}

/// Rule-based reflection provider.
pub struct RulesReflectionProvider {
    rules: Vec<Rule>,
}

impl RulesReflectionProvider {
    /// Creates a rule-based provider.
    #[must_use]
    pub fn new(rules: Vec<Rule>) -> Self {
        Self { rules }
    }
}

impl ReflectionProvider for RulesReflectionProvider {
    fn review(&self, _tenant: &TenantCtx, request: ReviewRequest) -> GResult<ReviewOutcome> {
        let mut findings = Vec::new();
        for rule in &self.rules {
            apply_rule(rule, &request.subject, &mut findings);
        }

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

fn apply_rule(rule: &Rule, subject: &Value, findings: &mut Vec<ReviewFinding>) {
    match rule {
        Rule::Exists {
            path,
            code,
            message,
        } => {
            if get_path(subject, path).is_none() {
                findings.push(ReviewFinding {
                    code: code.clone(),
                    message: message.clone(),
                });
            }
        }
        Rule::Contains {
            path,
            needle,
            code,
            message,
        } => {
            let contains = get_path(subject, path)
                .map(stringify_value)
                .map(|value| value.contains(needle))
                .unwrap_or(false);
            if !contains {
                findings.push(ReviewFinding {
                    code: code.clone(),
                    message: message.clone(),
                });
            }
        }
        Rule::CountGte {
            path,
            min,
            code,
            message,
        } => {
            let count = get_path(subject, path).map(count_value).unwrap_or(0);
            if count < *min {
                findings.push(ReviewFinding {
                    code: code.clone(),
                    message: message.clone(),
                });
            }
        }
        Rule::ScoreGte {
            path,
            min,
            code,
            message,
        } => {
            let score = get_path(subject, path)
                .and_then(Value::as_f64)
                .unwrap_or(f64::NEG_INFINITY);
            if score < *min {
                findings.push(ReviewFinding {
                    code: code.clone(),
                    message: message.clone(),
                });
            }
        }
        Rule::SchemaValid {
            path,
            schema,
            code,
            message,
        } => {
            let Some(value) = get_path(subject, path) else {
                findings.push(ReviewFinding {
                    code: code.clone(),
                    message: message.clone(),
                });
                return;
            };
            if !schema_valid(value, schema) {
                findings.push(ReviewFinding {
                    code: code.clone(),
                    message: message.clone(),
                });
            }
        }
    }
}

fn get_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    if path == "$" || path.is_empty() {
        return Some(value);
    }

    path.trim_start_matches("$.")
        .split('.')
        .try_fold(value, |current, segment| {
            if segment.is_empty() {
                return Some(current);
            }
            current.as_object()?.get(segment)
        })
}

fn stringify_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        _ => value.to_string(),
    }
}

fn count_value(value: &Value) -> usize {
    match value {
        Value::Array(values) => values.len(),
        Value::Object(values) => values.len(),
        Value::String(text) => text.len(),
        _ => 0,
    }
}

fn schema_valid(subject: &Value, schema: &Value) -> bool {
    if let Some(expected_type) = schema.get("type").and_then(Value::as_str) {
        match expected_type {
            "object" if !subject.is_object() => return false,
            "array" if !subject.is_array() => return false,
            "string" if !subject.is_string() => return false,
            "number" if !subject.is_number() => return false,
            "integer" if !(subject.as_i64().is_some() || subject.as_u64().is_some()) => {
                return false;
            }
            "boolean" if !subject.is_boolean() => return false,
            "null" if !subject.is_null() => return false,
            _ => {}
        }
    }
    true
}
