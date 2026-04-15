#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Shared reflection contract and typed review outcome models.

use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Request passed to a reflection provider.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReviewRequest {
    /// Stable review request identifier.
    pub request_id: String,
    /// Artifact or output under review.
    pub subject: Value,
    /// Optional schema target for deterministic validation.
    pub schema: Option<Value>,
    /// Free-form request metadata.
    pub metadata: Value,
}

impl ReviewRequest {
    /// Creates a new review request.
    pub fn new(request_id: impl Into<String>, subject: Value) -> GResult<Self> {
        let request_id = request_id.into();
        if request_id.trim().is_empty() {
            return Err(invalid("review request id must not be empty"));
        }
        Ok(Self {
            request_id,
            subject,
            schema: None,
            metadata: Value::Object(Map::new()),
        })
    }

    /// Attaches a schema target to the review request.
    #[must_use]
    pub fn with_schema(mut self, schema: Value) -> Self {
        self.schema = Some(schema);
        self
    }
}

/// A normalized reflection finding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewFinding {
    /// Machine-readable finding code.
    pub code: String,
    /// Human-readable finding detail.
    pub message: String,
}

impl ReviewFinding {
    /// Creates a validated review finding.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> GResult<Self> {
        let code = code.into();
        let message = message.into();
        if code.trim().is_empty() {
            return Err(invalid("review finding code must not be empty"));
        }
        if message.trim().is_empty() {
            return Err(invalid("review finding message must not be empty"));
        }
        Ok(Self { code, message })
    }
}

/// Top-level review outcome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewDisposition {
    /// The output is acceptable as-is.
    Accept,
    /// The output requires revisions.
    Revise,
    /// The output must be rejected.
    Reject,
}

/// Result produced by a reflection provider.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReviewOutcome {
    /// Final disposition.
    pub disposition: ReviewDisposition,
    /// Supporting findings.
    pub findings: Vec<ReviewFinding>,
}

impl ReviewOutcome {
    /// Builds an accepted outcome.
    #[must_use]
    pub fn accept() -> Self {
        Self {
            disposition: ReviewDisposition::Accept,
            findings: Vec::new(),
        }
    }

    /// Builds a revise outcome with no findings yet.
    #[must_use]
    pub fn revise() -> Self {
        Self {
            disposition: ReviewDisposition::Revise,
            findings: Vec::new(),
        }
    }

    /// Builds a reject outcome with no findings yet.
    #[must_use]
    pub fn reject() -> Self {
        Self {
            disposition: ReviewDisposition::Reject,
            findings: Vec::new(),
        }
    }

    /// Attaches a finding to the outcome.
    #[must_use]
    pub fn with_finding(mut self, finding: ReviewFinding) -> Self {
        self.findings.push(finding);
        self
    }

    /// Validates the typed review outcome.
    pub fn validate(&self) -> GResult<()> {
        for finding in &self.findings {
            if finding.code.trim().is_empty() {
                return Err(invalid("review finding code must not be empty"));
            }
            if finding.message.trim().is_empty() {
                return Err(invalid("review finding message must not be empty"));
            }
        }
        Ok(())
    }
}

/// Contract implemented by reflection backends.
pub trait ReflectionProvider: Send + Sync {
    /// Reviews a structured subject and returns a typed outcome.
    fn review(&self, tenant: &TenantCtx, request: ReviewRequest) -> GResult<ReviewOutcome>;
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::{ReviewFinding, ReviewOutcome, ReviewRequest};
    use serde_json::json;

    #[test]
    fn request_requires_id() {
        assert!(ReviewRequest::new("", json!({"value": 1})).is_err());
    }

    #[test]
    fn outcome_accumulates_findings() {
        let outcome = ReviewOutcome::accept().with_finding(ReviewFinding {
            code: "schema.warning".to_string(),
            message: "Optional field missing".to_string(),
        });
        assert_eq!(outcome.findings.len(), 1);
    }
}
