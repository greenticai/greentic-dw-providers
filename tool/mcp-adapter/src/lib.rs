#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! MCP-backed tool adapter.

use greentic_dw_providers_common::{ToolVariant, tool_pack_manifest, tool_provider_decl};
use greentic_dw_tool::{ToolAdapter, ToolCallRequest, ToolCallResult, ToolDescriptor};
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use rmcp::{
    serve_client,
    service::{RoleClient, RunningService},
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::sync::Mutex;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::runtime::{Builder, Runtime};

/// Static adapter metadata for an MCP connection.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct McpAdapterConfig {
    /// Human-readable name used in metadata.
    pub adapter_name: String,
}

impl Default for McpAdapterConfig {
    fn default() -> Self {
        Self {
            adapter_name: "mcp-adapter".to_string(),
        }
    }
}

/// Tool adapter that forwards discovery and invocation to a normal MCP server.
pub struct McpToolAdapter {
    config: McpAdapterConfig,
    runtime: Runtime,
    client: Mutex<RunningService<RoleClient, ()>>,
}

impl McpToolAdapter {
    /// Connects the adapter to an MCP transport implemented as async read/write streams.
    pub fn connect<T>(config: McpAdapterConfig, transport: T) -> GResult<Self>
    where
        T: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .map_err(internal_error)?;
        let client = runtime
            .block_on(serve_client((), transport))
            .map_err(internal_error)?;

        Ok(Self {
            config,
            runtime,
            client: Mutex::new(client),
        })
    }

    /// Returns the canonical provider declaration for this backend.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        tool_provider_decl(ToolVariant::McpAdapter)
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.tool.mcp-adapter")?;
        tool_pack_manifest(pack_id, ToolVariant::McpAdapter)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }

    /// Gracefully closes the MCP connection.
    pub fn close(&self) -> GResult<()> {
        let mut client = self.client.lock().map_err(lock_error)?;
        self.runtime
            .block_on(async { client.close().await })
            .map_err(internal_error)?;
        Ok(())
    }
}

impl ToolAdapter for McpToolAdapter {
    fn describe(&self, _tenant: &TenantCtx) -> GResult<Vec<ToolDescriptor>> {
        let client = self.client.lock().map_err(lock_error)?;
        let tools = self
            .runtime
            .block_on(client.peer().list_all_tools())
            .map_err(internal_error)?;

        tools.into_iter().map(map_tool_descriptor).collect()
    }

    fn invoke(&self, _tenant: &TenantCtx, request: ToolCallRequest) -> GResult<ToolCallResult> {
        let client = self.client.lock().map_err(lock_error)?;
        let params = rmcp::model::CallToolRequestParams::new(request.tool_name.clone())
            .with_arguments(normalize_arguments(request.arguments));
        let result = self
            .runtime
            .block_on(client.peer().call_tool(params))
            .map_err(internal_error)?;

        let value = result.structured_content.unwrap_or_else(|| {
            serde_json::to_value(&result.content).unwrap_or_else(|_| Value::Array(Vec::new()))
        });

        Ok(ToolCallResult {
            value,
            is_error: result.is_error.unwrap_or(false),
            metadata: json!({
                "adapter": self.config.adapter_name,
                "content_items": result.content.len()
            }),
        })
    }
}

fn map_tool_descriptor(tool: rmcp::model::Tool) -> GResult<ToolDescriptor> {
    let mut descriptor = ToolDescriptor::new(tool.name.into_owned())?
        .with_input_schema(Value::Object((*tool.input_schema).clone()));

    if let Some(title) = tool.title {
        descriptor = descriptor.with_title(title);
    }
    if let Some(description) = tool.description {
        descriptor = descriptor.with_description(description.into_owned());
    }
    if let Some(output_schema) = tool.output_schema {
        descriptor = descriptor.with_output_schema(Value::Object((*output_schema).clone()));
    }

    Ok(descriptor)
}

fn normalize_arguments(arguments: Value) -> rmcp::model::JsonObject {
    match arguments {
        Value::Null => Map::new(),
        Value::Object(map) => map,
        value => {
            let mut map = Map::new();
            map.insert("input".to_string(), value);
            map
        }
    }
}

fn internal_error(err: impl std::fmt::Display) -> GreenticError {
    GreenticError::new(ErrorCode::Internal, err.to_string())
}

fn lock_error(err: impl std::fmt::Display) -> GreenticError {
    GreenticError::new(
        ErrorCode::Internal,
        format!("mcp adapter lock poisoned: {err}"),
    )
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{McpToolAdapter, map_tool_descriptor, normalize_arguments};
    use rmcp::model::Tool;
    use serde_json::{Value, json};

    #[test]
    fn scalar_arguments_are_wrapped() {
        let args = normalize_arguments(json!("hello"));
        assert_eq!(args.get("input"), Some(&json!("hello")));
    }

    #[test]
    fn object_arguments_are_passed_through() {
        let args = normalize_arguments(json!({"value": 1}));
        assert_eq!(args.get("value"), Some(&json!(1)));
    }

    #[test]
    fn null_arguments_become_empty_object() {
        let args = normalize_arguments(Value::Null);
        assert!(args.is_empty());
    }

    #[test]
    fn rmcp_tool_maps_into_shared_descriptor() {
        let tool: Tool = serde_json::from_value(json!({
            "name": "echo",
            "title": "Echo",
            "description": "Repeats input",
            "inputSchema": {},
            "outputSchema": {}
        }))
        .expect("tool");
        let descriptor = map_tool_descriptor(tool).expect("descriptor");

        assert_eq!(descriptor.name, "echo");
        assert_eq!(descriptor.title.as_deref(), Some("Echo"));
        assert_eq!(
            descriptor.input_schema,
            Value::Object(serde_json::Map::new())
        );
    }

    #[test]
    fn provider_manifest_uses_mcp_pack_id() {
        let manifest = McpToolAdapter::pack_manifest().expect("manifest");
        assert_eq!(
            manifest.pack_id.as_str(),
            "greentic.dw.providers.tool.mcp-adapter"
        );
        assert_eq!(
            McpToolAdapter::provider_decl().provider_type,
            "dw.tool.mcp-adapter"
        );
    }
}
