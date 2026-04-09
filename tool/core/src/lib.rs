#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Shared tool adapter contract and request models.

use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Describes a callable tool exposed by an adapter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolDescriptor {
    /// Stable tool name.
    pub name: String,
    /// Optional human-readable title.
    pub title: Option<String>,
    /// Optional human-readable description.
    pub description: Option<String>,
    /// JSON schema describing the input payload.
    pub input_schema: Value,
    /// Optional JSON schema describing the output payload.
    pub output_schema: Option<Value>,
}

impl ToolDescriptor {
    /// Creates a new descriptor with an empty object input schema.
    pub fn new(name: impl Into<String>) -> GResult<Self> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(invalid("tool descriptor name must not be empty"));
        }

        Ok(Self {
            name,
            title: None,
            description: None,
            input_schema: Value::Object(Map::new()),
            output_schema: None,
        })
    }

    /// Sets a human-readable title.
    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets a human-readable description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the input schema.
    #[must_use]
    pub fn with_input_schema(mut self, input_schema: Value) -> Self {
        self.input_schema = input_schema;
        self
    }

    /// Sets the output schema.
    #[must_use]
    pub fn with_output_schema(mut self, output_schema: Value) -> Self {
        self.output_schema = Some(output_schema);
        self
    }
}

/// Request passed to a tool adapter invocation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolCallRequest {
    /// Name of the tool to invoke.
    pub tool_name: String,
    /// Arbitrary input payload.
    pub arguments: Value,
}

impl ToolCallRequest {
    /// Creates a new call request.
    pub fn new(tool_name: impl Into<String>, arguments: Value) -> GResult<Self> {
        let tool_name = tool_name.into();
        if tool_name.trim().is_empty() {
            return Err(invalid("tool name must not be empty"));
        }

        Ok(Self {
            tool_name,
            arguments,
        })
    }
}

/// Result produced by a tool adapter invocation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToolCallResult {
    /// Structured result payload.
    pub value: Value,
    /// Whether the invocation represents an error response.
    pub is_error: bool,
    /// Optional execution metadata.
    pub metadata: Value,
}

impl ToolCallResult {
    /// Builds a successful tool result.
    #[must_use]
    pub fn success(value: Value) -> Self {
        Self {
            value,
            is_error: false,
            metadata: Value::Object(Map::new()),
        }
    }

    /// Builds an error tool result.
    #[must_use]
    pub fn error(value: Value) -> Self {
        Self {
            value,
            is_error: true,
            metadata: Value::Object(Map::new()),
        }
    }

    /// Attaches execution metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Contract implemented by tool backends.
pub trait ToolAdapter: Send + Sync {
    /// Describes the tools made available by this adapter.
    fn describe(&self, tenant: &TenantCtx) -> GResult<Vec<ToolDescriptor>>;

    /// Invokes one of the adapter's tools.
    fn invoke(&self, tenant: &TenantCtx, request: ToolCallRequest) -> GResult<ToolCallResult>;
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}

#[cfg(test)]
mod tests {
    use super::{ToolCallRequest, ToolCallResult, ToolDescriptor};
    use serde_json::json;

    #[test]
    fn descriptor_requires_name() {
        let err = match ToolDescriptor::new("   ") {
            Ok(_) => panic!("empty name should be invalid"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn request_requires_tool_name() {
        let err = match ToolCallRequest::new("", json!({"x": 1})) {
            Ok(_) => panic!("tool name should be invalid"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("tool name"));
    }

    #[test]
    fn descriptor_builders_preserve_fields() {
        let descriptor = match ToolDescriptor::new("echo") {
            Ok(descriptor) => descriptor,
            Err(err) => panic!("descriptor should be valid: {err}"),
        }
        .with_title("Echo")
        .with_description("Repeats input")
        .with_input_schema(json!({"type": "object"}))
        .with_output_schema(json!({"type": "string"}));

        assert_eq!(descriptor.name, "echo");
        assert_eq!(descriptor.title.as_deref(), Some("Echo"));
        assert_eq!(descriptor.description.as_deref(), Some("Repeats input"));
        assert_eq!(descriptor.input_schema, json!({"type": "object"}));
        assert_eq!(descriptor.output_schema, Some(json!({"type": "string"})));
    }

    #[test]
    fn request_accepts_structured_arguments() {
        let request = match ToolCallRequest::new("echo", json!({"value": 1})) {
            Ok(request) => request,
            Err(err) => panic!("request should be valid: {err}"),
        };
        assert_eq!(request.tool_name, "echo");
        assert_eq!(request.arguments, json!({"value": 1}));
    }

    #[test]
    fn result_helpers_mark_success_and_error_states() {
        let success =
            ToolCallResult::success(json!({"ok": true})).with_metadata(json!({"source": "test"}));
        let error = ToolCallResult::error(json!({"message": "bad"}));

        assert!(!success.is_error);
        assert_eq!(success.metadata, json!({"source": "test"}));
        assert!(error.is_error);
        assert_eq!(error.metadata, json!({}));
    }
}
