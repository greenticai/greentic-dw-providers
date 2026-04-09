#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Greentic component-backed tool adapter.

use greentic_component::test_harness::{HarnessConfig, TestHarness};
use greentic_dw_providers_common::{ToolVariant, tool_pack_manifest, tool_provider_decl};
use greentic_dw_tool::{ToolAdapter, ToolCallRequest, ToolCallResult, ToolDescriptor};
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

/// Configuration for the component-backed tool adapter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComponentAdapterConfig {
    /// Tool name exposed by this adapter.
    pub tool_name: String,
    /// Optional title shown in discovery results.
    pub title: Option<String>,
    /// Optional description shown in discovery results.
    pub description: Option<String>,
    /// Input schema published for the tool.
    pub input_schema: Value,
    /// Output schema published for the tool.
    pub output_schema: Option<Value>,
    /// Path to the Greentic component WASM artifact.
    pub component_wasm_path: PathBuf,
    /// Operation to invoke on the component.
    pub operation: String,
    /// Flow identifier passed to the component exec context.
    pub flow_id: String,
    /// Optional node id passed to the component exec context.
    pub node_id: Option<String>,
    /// Prefix used for harness-managed state scope.
    pub state_prefix: String,
    /// Optional component config JSON.
    pub component_config: Option<Value>,
    /// Whether state reads are allowed.
    pub allow_state_read: bool,
    /// Whether state writes are allowed.
    pub allow_state_write: bool,
    /// Whether state deletes are allowed.
    pub allow_state_delete: bool,
    /// Whether HTTP access is allowed.
    pub allow_http: bool,
    /// Invocation timeout in milliseconds.
    pub timeout_ms: u64,
    /// Max memory budget passed to the harness.
    pub max_memory_bytes: usize,
}

impl Default for ComponentAdapterConfig {
    fn default() -> Self {
        Self {
            tool_name: "component-tool".to_string(),
            title: Some("Component Tool".to_string()),
            description: Some("Invokes a configured Greentic component operation.".to_string()),
            input_schema: Value::Object(Map::new()),
            output_schema: None,
            component_wasm_path: PathBuf::from("component.wasm"),
            operation: "tool.invoke".to_string(),
            flow_id: "dw-tool-component".to_string(),
            node_id: Some("component-adapter".to_string()),
            state_prefix: "dw:tool:component".to_string(),
            component_config: None,
            allow_state_read: true,
            allow_state_write: true,
            allow_state_delete: false,
            allow_http: false,
            timeout_ms: 5_000,
            max_memory_bytes: 64 * 1024 * 1024,
        }
    }
}

/// Adapter that proxies one configured Greentic component operation as a tool.
pub struct ComponentToolAdapter {
    config: ComponentAdapterConfig,
    wasm_bytes: Vec<u8>,
}

impl ComponentToolAdapter {
    /// Builds the adapter and loads the configured component bytes.
    pub fn new(config: ComponentAdapterConfig) -> GResult<Self> {
        if config.tool_name.trim().is_empty() {
            return Err(invalid("component adapter tool name must not be empty"));
        }
        if config.operation.trim().is_empty() {
            return Err(invalid("component adapter operation must not be empty"));
        }
        if config.flow_id.trim().is_empty() {
            return Err(invalid("component adapter flow id must not be empty"));
        }
        if config.state_prefix.trim().is_empty() {
            return Err(invalid("component adapter state prefix must not be empty"));
        }

        let wasm_bytes = fs::read(&config.component_wasm_path).map_err(|err| {
            GreenticError::new(
                ErrorCode::InvalidInput,
                format!(
                    "failed to read component wasm at {}: {err}",
                    config.component_wasm_path.display()
                ),
            )
        })?;

        Ok(Self { config, wasm_bytes })
    }

    /// Returns the canonical provider declaration for this backend.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        tool_provider_decl(ToolVariant::ComponentAdapter)
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.tool.component-adapter")?;
        tool_pack_manifest(pack_id, ToolVariant::ComponentAdapter)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }

    fn descriptor(&self) -> GResult<ToolDescriptor> {
        let mut descriptor = ToolDescriptor::new(self.config.tool_name.clone())?;
        if let Some(title) = &self.config.title {
            descriptor = descriptor.with_title(title.clone());
        }
        if let Some(description) = &self.config.description {
            descriptor = descriptor.with_description(description.clone());
        }
        descriptor = descriptor.with_input_schema(self.config.input_schema.clone());
        if let Some(output_schema) = &self.config.output_schema {
            descriptor = descriptor.with_output_schema(output_schema.clone());
        }
        Ok(descriptor)
    }
}

impl ToolAdapter for ComponentToolAdapter {
    fn describe(&self, _tenant: &TenantCtx) -> GResult<Vec<ToolDescriptor>> {
        Ok(vec![self.descriptor()?])
    }

    fn invoke(&self, tenant: &TenantCtx, request: ToolCallRequest) -> GResult<ToolCallResult> {
        if request.tool_name != self.config.tool_name {
            return Err(invalid(format!(
                "unknown component tool `{}`; expected `{}`",
                request.tool_name, self.config.tool_name
            )));
        }

        let harness = TestHarness::new(HarnessConfig {
            wasm_bytes: self.wasm_bytes.clone(),
            tenant_ctx: tenant.clone(),
            flow_id: self.config.flow_id.clone(),
            node_id: self.config.node_id.clone(),
            state_prefix: self.config.state_prefix.clone(),
            state_seeds: Vec::new(),
            allow_state_read: self.config.allow_state_read,
            allow_state_write: self.config.allow_state_write,
            allow_state_delete: self.config.allow_state_delete,
            allow_secrets: false,
            allowed_secrets: HashSet::new(),
            secrets: HashMap::new(),
            wasi_preopens: Vec::new(),
            config: self.config.component_config.clone(),
            allow_http: self.config.allow_http,
            timeout_ms: self.config.timeout_ms,
            max_memory_bytes: self.config.max_memory_bytes,
        })
        .map_err(internal_error)?;

        let outcome = harness
            .invoke(&self.config.operation, &request.arguments)
            .map_err(internal_error)?;

        let value = serde_json::from_str::<Value>(&outcome.output_json)
            .unwrap_or_else(|_| Value::String(outcome.output_json.clone()));

        Ok(ToolCallResult::success(value).with_metadata(json!({
            "adapter": "component-adapter",
            "component_wasm_path": self.config.component_wasm_path.display().to_string(),
            "operation": self.config.operation,
            "instantiate_ms": outcome.instantiate_ms,
            "run_ms": outcome.run_ms
        })))
    }
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}

fn internal_error(err: impl std::fmt::Display) -> GreenticError {
    GreenticError::new(ErrorCode::Internal, err.to_string())
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{ComponentAdapterConfig, ComponentToolAdapter};
    use greentic_dw_tool::{ToolAdapter, ToolCallRequest};
    use greentic_types::{EnvId, TenantCtx, TenantId};
    use serde_json::json;
    use std::path::PathBuf;

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-tool").expect("tenant id"),
        )
    }

    fn sample_existing_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")
    }

    #[test]
    fn describe_exposes_single_configured_tool() {
        let config = ComponentAdapterConfig {
            component_wasm_path: sample_existing_path(),
            tool_name: "demo-component".to_string(),
            ..ComponentAdapterConfig::default()
        };

        let adapter = ComponentToolAdapter::new(config).expect("adapter");
        let tools = adapter.describe(&tenant()).expect("describe");

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "demo-component");
    }

    #[test]
    fn provider_manifest_uses_component_pack_id() {
        let manifest = ComponentToolAdapter::pack_manifest().expect("manifest");
        assert_eq!(
            manifest.pack_id.as_str(),
            "greentic.dw.providers.tool.component-adapter"
        );
        assert_eq!(
            ComponentToolAdapter::provider_decl().provider_type,
            "dw.tool.component-adapter"
        );
    }

    #[test]
    fn new_rejects_empty_operation() {
        let config = ComponentAdapterConfig {
            component_wasm_path: sample_existing_path(),
            operation: "   ".to_string(),
            ..ComponentAdapterConfig::default()
        };

        let err = match ComponentToolAdapter::new(config) {
            Ok(_) => panic!("operation should be invalid"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("operation"));
    }

    #[test]
    fn invoke_rejects_unknown_tool_name_before_harness_start() {
        let config = ComponentAdapterConfig {
            component_wasm_path: sample_existing_path(),
            tool_name: "demo-component".to_string(),
            ..ComponentAdapterConfig::default()
        };

        let adapter = ComponentToolAdapter::new(config).expect("adapter");
        let err = adapter
            .invoke(
                &tenant(),
                ToolCallRequest::new("other-tool", json!({"value": 1})).expect("request"),
            )
            .expect_err("tool mismatch invalid");

        assert!(err.to_string().contains("unknown component tool"));
    }
}
