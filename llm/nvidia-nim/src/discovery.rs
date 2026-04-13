use greentic_dw_llm::{LlmError, LlmResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::NvidiaNimError;

/// Discovered model entry returned by `/v1/models`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NvidiaNimModel {
    /// Stable model identifier.
    pub id: String,
    /// Optional owner or source metadata.
    pub owned_by: Option<String>,
}

/// Parsed health check response.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NvidiaNimHealth {
    /// Whether the endpoint is healthy.
    pub ok: bool,
    /// Optional raw status text.
    pub status: Option<String>,
}

/// Parses a model listing response from `/v1/models`.
pub fn parse_models_response(value: &Value) -> LlmResult<Vec<NvidiaNimModel>> {
    let data = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| LlmError::from(NvidiaNimError::discovery("models response missing data")))?;

    Ok(data
        .iter()
        .filter_map(|entry| {
            entry
                .get("id")
                .and_then(Value::as_str)
                .map(|id| NvidiaNimModel {
                    id: id.to_string(),
                    owned_by: entry
                        .get("owned_by")
                        .and_then(Value::as_str)
                        .map(ToOwned::to_owned),
                })
        })
        .collect())
}

/// Parses a health response from `/health/ready` or `/health/live`.
pub fn parse_health_response(value: &Value) -> LlmResult<NvidiaNimHealth> {
    if let Some(ok) = value.get("ok").and_then(Value::as_bool) {
        return Ok(NvidiaNimHealth {
            ok,
            status: value
                .get("status")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
        });
    }
    if let Some(status) = value.get("status").and_then(Value::as_str) {
        return Ok(NvidiaNimHealth {
            ok: matches!(status, "ok" | "ready" | "live" | "healthy"),
            status: Some(status.to_string()),
        });
    }
    Err(LlmError::from(NvidiaNimError::health(
        "health response missing status",
    )))
}
