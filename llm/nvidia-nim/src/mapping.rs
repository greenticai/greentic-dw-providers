use greentic_dw_llm::{
    LlmContentPart, LlmError, LlmErrorKind, LlmFinishReason, LlmMessage, LlmMessageRole,
    LlmRequest, LlmResponse, LlmResult, LlmToolCall, LlmToolChoice, LlmUsage,
};
use serde_json::{Map, Value, json};

use crate::NvidiaNimError;
use crate::config::NvidiaNimConfig;

pub(crate) fn build_request(config: &NvidiaNimConfig, request: &LlmRequest) -> LlmResult<Value> {
    let mut body = Map::new();
    body.insert(
        "model".to_string(),
        Value::String(
            request
                .model
                .clone()
                .unwrap_or_else(|| config.model.clone()),
        ),
    );
    body.insert(
        "messages".to_string(),
        Value::Array(
            request
                .messages
                .iter()
                .map(map_chat_message)
                .collect::<LlmResult<Vec<_>>>()?,
        ),
    );
    body.insert("stream".to_string(), Value::Bool(request.stream));

    if !request.tools.is_empty() {
        body.insert(
            "tools".to_string(),
            Value::Array(
                request
                    .tools
                    .iter()
                    .map(|tool| {
                        json!({
                            "type": "function",
                            "function": {
                                "name": tool.name,
                                "description": tool.description,
                                "parameters": tool.input_schema,
                            }
                        })
                    })
                    .collect(),
            ),
        );
    }
    if !matches!(request.tool_choice, LlmToolChoice::None) {
        body.insert(
            "tool_choice".to_string(),
            map_tool_choice(&request.tool_choice),
        );
    }
    if config.allow_structured_outputs
        && let Some(structured_output) = &request.structured_output
    {
        body.insert(
            "response_format".to_string(),
            json!({
                "type": "json_schema",
                "json_schema": {
                    "name": structured_output.name,
                    "description": structured_output.description,
                    "schema": structured_output.schema,
                    "strict": structured_output.strict,
                }
            }),
        );
    }
    Ok(Value::Object(body))
}

fn map_chat_message(message: &LlmMessage) -> LlmResult<Value> {
    let content = message
        .parts
        .iter()
        .map(|part| match part {
            LlmContentPart::Text { text } => Ok(text.clone()),
            LlmContentPart::Json { value } => Ok(value.to_string()),
            LlmContentPart::ImageUrl { url, .. } => Ok(url.clone()),
            LlmContentPart::ToolResult { .. } => Err(LlmError::invalid_request(
                "tool results are not supported in NIM chat content mapping",
            )),
        })
        .collect::<LlmResult<Vec<_>>>()?
        .join("\n");

    Ok(json!({
        "role": role_name(message.role),
        "content": content,
    }))
}

fn role_name(role: LlmMessageRole) -> &'static str {
    match role {
        LlmMessageRole::System => "system",
        LlmMessageRole::User => "user",
        LlmMessageRole::Assistant => "assistant",
        LlmMessageRole::Tool => "tool",
    }
}

fn map_tool_choice(choice: &LlmToolChoice) -> Value {
    match choice {
        LlmToolChoice::Auto => json!("auto"),
        LlmToolChoice::None => json!("none"),
        LlmToolChoice::Required => json!("required"),
        LlmToolChoice::Tool(name) => json!({
            "type": "function",
            "function": { "name": name }
        }),
    }
}

pub(crate) fn parse_response(value: &Value, request: &LlmRequest) -> LlmResult<LlmResponse> {
    let choice = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .ok_or_else(|| LlmError::from(NvidiaNimError::inference("response missing choices")))?;

    let message = choice.get("message").cloned().unwrap_or(Value::Null);
    let mut output = Vec::new();
    let mut tool_calls = Vec::new();

    if let Some(content) = message.get("content").and_then(Value::as_str)
        && !content.is_empty()
    {
        output.push(LlmMessage {
            role: LlmMessageRole::Assistant,
            parts: vec![LlmContentPart::Text {
                text: content.to_string(),
            }],
            name: None,
        });
    }
    if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
        for call in calls {
            let arguments = call
                .get("function")
                .and_then(|value| value.get("arguments"))
                .and_then(Value::as_str)
                .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
                .unwrap_or(Value::Null);
            tool_calls.push(LlmToolCall {
                id: call
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                name: call
                    .get("function")
                    .and_then(|value| value.get("name"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                arguments,
            });
        }
    }

    let mut response = LlmResponse::new(output);
    response.model = value
        .get("model")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    response.tool_calls = tool_calls;
    response.finish_reason = choice
        .get("finish_reason")
        .and_then(Value::as_str)
        .map(|reason| match reason {
            "stop" => LlmFinishReason::Stop,
            "length" => LlmFinishReason::Length,
            "tool_calls" => LlmFinishReason::ToolCalls,
            other => LlmFinishReason::Other(other.to_string()),
        });
    response.usage = value.get("usage").and_then(|usage| {
        Some(LlmUsage {
            input_tokens: usage.get("prompt_tokens")?.as_u64()?,
            output_tokens: usage.get("completion_tokens")?.as_u64()?,
            total_tokens: usage.get("total_tokens")?.as_u64()?,
        })
    });
    if request.structured_output.is_some() {
        let text = response.text_output();
        if !text.is_empty() {
            response.structured_output = serde_json::from_str(&text).ok();
        }
    }
    response.metadata = value.clone();
    Ok(response)
}

pub(crate) fn map_http_error(status: u16, body: &str, endpoint: &'static str) -> LlmError {
    let parsed = serde_json::from_str::<Value>(body).ok();
    let message = parsed
        .as_ref()
        .and_then(|value| value.get("error"))
        .and_then(|value| value.get("message"))
        .and_then(Value::as_str)
        .unwrap_or(body)
        .to_string();
    let prefixed = match endpoint {
        "inference" => NvidiaNimError::inference(message).to_string(),
        "discovery" => NvidiaNimError::discovery(message).to_string(),
        "health" => NvidiaNimError::health(message).to_string(),
        _ => format!("NIM provider request failed: {message}"),
    };

    match status {
        400 => LlmError::new(LlmErrorKind::InvalidRequest, prefixed),
        401 | 403 => LlmError::new(LlmErrorKind::Auth, prefixed),
        404 | 422 => LlmError::new(LlmErrorKind::UnsupportedModel, prefixed),
        408 => LlmError::new(LlmErrorKind::Timeout, prefixed).retryable(true),
        429 => LlmError::new(LlmErrorKind::RateLimited, prefixed).retryable(true),
        500..=599 => LlmError::new(LlmErrorKind::Provider, prefixed).retryable(true),
        _ => LlmError::new(LlmErrorKind::Provider, prefixed),
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{build_request, map_chat_message, map_http_error, parse_response};
    use crate::config::NvidiaNimConfig;
    use greentic_dw_llm::{
        LlmContentPart, LlmErrorKind, LlmMessage, LlmMessageRole, LlmRequest,
        LlmStructuredOutputSpec,
    };
    use serde_json::{Value, json};

    fn config() -> NvidiaNimConfig {
        let mut config = NvidiaNimConfig::new("https://nim.example/v1", "meta/llama", 5_000);
        config.allow_structured_outputs = true;
        config
    }

    #[test]
    fn map_chat_message_rejects_tool_results() {
        let message = LlmMessage {
            role: LlmMessageRole::Tool,
            parts: vec![LlmContentPart::ToolResult {
                tool_name: "lookup".to_string(),
                tool_call_id: "call_1".to_string(),
                result: Value::Null,
                is_error: false,
            }],
            name: None,
        };
        let err = map_chat_message(&message).expect_err("unsupported content");
        assert_eq!(err.kind, LlmErrorKind::InvalidRequest);
    }

    #[test]
    fn build_request_maps_required_tool_choice() {
        let request = LlmRequest::new(
            "req-1",
            vec![LlmMessage::text(LlmMessageRole::User, "hello").expect("message")],
        )
        .expect("request")
        .with_tool_choice(greentic_dw_llm::LlmToolChoice::Required);
        let body = build_request(&config(), &request).expect("body");
        assert_eq!(body["tool_choice"], json!("required"));
    }

    #[test]
    fn parse_response_rejects_missing_choices() {
        let request = LlmRequest::new(
            "req-2",
            vec![LlmMessage::text(LlmMessageRole::User, "hi").expect("message")],
        )
        .expect("request");
        let err = parse_response(&json!({}), &request).expect_err("missing choices");
        assert_eq!(err.kind, LlmErrorKind::Provider);
        assert!(err.message.contains("NIM inference request failed"));
    }

    #[test]
    fn parse_response_leaves_invalid_structured_output_unset() {
        let request = LlmRequest::new(
            "req-3",
            vec![LlmMessage::text(LlmMessageRole::User, "hi").expect("message")],
        )
        .expect("request")
        .with_structured_output(
            LlmStructuredOutputSpec::new("result", json!({"type":"object"})).expect("schema"),
        );
        let response = parse_response(
            &json!({
                "choices": [{
                    "finish_reason": "stop",
                    "message": { "content": "not json" }
                }]
            }),
            &request,
        )
        .expect("response");
        assert!(response.structured_output.is_none());
    }

    #[test]
    fn unknown_http_endpoint_uses_generic_prefix() {
        let err = map_http_error(500, "boom", "other");
        assert_eq!(err.kind, LlmErrorKind::Provider);
        assert!(err.message.contains("NIM provider request failed"));
    }
}
