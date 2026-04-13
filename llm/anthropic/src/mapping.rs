use greentic_dw_llm::{
    LlmContentPart, LlmConversationState, LlmError, LlmErrorKind, LlmFinishReason, LlmMessage,
    LlmMessageRole, LlmRequest, LlmResponse, LlmResult, LlmToolCall, LlmToolChoice, LlmUsage,
};
use serde_json::{Map, Value, json};

use crate::AnthropicError;
use crate::config::AnthropicConfig;

const SYNTHETIC_STRUCTURED_TOOL_NAME: &str = "emit_structured_output";

pub(crate) fn build_messages_request(
    config: &AnthropicConfig,
    request: &LlmRequest,
) -> LlmResult<Value> {
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
        "max_tokens".to_string(),
        Value::Number(u64::from(request.max_output_tokens.unwrap_or(config.max_tokens)).into()),
    );
    body.insert("stream".to_string(), Value::Bool(request.stream));

    if let Some(temperature) = request.temperature {
        body.insert("temperature".to_string(), json!(temperature));
    }
    if let Some(system) = build_system_blocks(request)? {
        body.insert("system".to_string(), system);
    }
    body.insert(
        "messages".to_string(),
        Value::Array(build_messages(request)?),
    );

    let mut tools = build_tools(request);
    let mut tool_choice = map_tool_choice(&request.tool_choice);
    if let Some(structured_output) = &request.structured_output {
        tools.push(json!({
            "name": structured_output.name,
            "description": structured_output.description,
            "input_schema": structured_output.schema,
        }));
        tool_choice = Some(json!({
            "type": "tool",
            "name": structured_output.name,
        }));
    }
    if !tools.is_empty() {
        body.insert("tools".to_string(), Value::Array(tools));
        if let Some(tool_choice) = tool_choice {
            body.insert("tool_choice".to_string(), tool_choice);
        }
    }

    if config.allow_thinking
        && let Some(budget_tokens) = config.thinking_budget_tokens
    {
        body.insert(
            "thinking".to_string(),
            json!({
                "type": "enabled",
                "budget_tokens": budget_tokens,
            }),
        );
    }

    Ok(Value::Object(body))
}

fn build_system_blocks(request: &LlmRequest) -> LlmResult<Option<Value>> {
    let mut blocks = Vec::new();
    for message in &request.messages {
        if message.role == LlmMessageRole::System {
            for part in &message.parts {
                match part {
                    LlmContentPart::Text { text } => blocks.push(json!({
                        "type": "text",
                        "text": text,
                    })),
                    LlmContentPart::Json { value } => blocks.push(json!({
                        "type": "text",
                        "text": value.to_string(),
                    })),
                    _ => {
                        return Err(LlmError::invalid_request(
                            "system messages only support text and json content for Anthropic",
                        ));
                    }
                }
            }
        }
    }
    if blocks.is_empty() {
        Ok(None)
    } else {
        Ok(Some(Value::Array(blocks)))
    }
}

fn build_messages(request: &LlmRequest) -> LlmResult<Vec<Value>> {
    let mut messages = Vec::new();
    for message in &request.messages {
        if message.role == LlmMessageRole::System {
            continue;
        }
        let role = match message.role {
            LlmMessageRole::Assistant => "assistant",
            LlmMessageRole::Tool | LlmMessageRole::User => "user",
            LlmMessageRole::System => unreachable!(),
        };
        messages.push(json!({
            "role": role,
            "content": map_message_parts(&message.parts)?,
        }));
    }
    Ok(messages)
}

fn map_message_parts(parts: &[LlmContentPart]) -> LlmResult<Vec<Value>> {
    parts
        .iter()
        .map(|part| match part {
            LlmContentPart::Text { text } => Ok(json!({ "type": "text", "text": text })),
            LlmContentPart::Json { value } => {
                Ok(json!({ "type": "text", "text": value.to_string() }))
            }
            LlmContentPart::ImageUrl { url, .. } => Ok(json!({
                "type": "image",
                "source": {
                    "type": "url",
                    "url": url,
                }
            })),
            LlmContentPart::ToolResult {
                tool_call_id,
                result,
                is_error,
                ..
            } => Ok(json!({
                "type": "tool_result",
                "tool_use_id": tool_call_id,
                "content": result.to_string(),
                "is_error": is_error,
            })),
        })
        .collect()
}

fn build_tools(request: &LlmRequest) -> Vec<Value> {
    request
        .tools
        .iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "input_schema": tool.input_schema,
            })
        })
        .collect()
}

fn map_tool_choice(choice: &LlmToolChoice) -> Option<Value> {
    match choice {
        LlmToolChoice::None => None,
        LlmToolChoice::Auto => Some(json!({ "type": "auto" })),
        LlmToolChoice::Required => Some(json!({ "type": "any" })),
        LlmToolChoice::Tool(name) => Some(json!({ "type": "tool", "name": name })),
    }
}

pub(crate) fn parse_messages_response(
    response: &Value,
    request: &LlmRequest,
) -> LlmResult<LlmResponse> {
    let mut output = Vec::new();
    let mut tool_calls = Vec::new();
    let mut structured_output = None;

    let content = response
        .get("content")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut text_parts = Vec::new();
    for block in content {
        match block.get("type").and_then(Value::as_str) {
            Some("text") => {
                if let Some(text) = block.get("text").and_then(Value::as_str) {
                    text_parts.push(LlmContentPart::Text {
                        text: text.to_string(),
                    });
                }
            }
            Some("tool_use") => {
                let name = block
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string();
                let arguments = block.get("input").cloned().unwrap_or(Value::Null);
                let tool_call = LlmToolCall {
                    id: block
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    name: name.clone(),
                    arguments: arguments.clone(),
                };
                if request.structured_output.is_some()
                    && name
                        == request
                            .structured_output
                            .as_ref()
                            .map(|v| v.name.as_str())
                            .unwrap_or(SYNTHETIC_STRUCTURED_TOOL_NAME)
                {
                    structured_output = Some(arguments);
                } else {
                    tool_calls.push(tool_call);
                }
            }
            _ => {}
        }
    }

    if !text_parts.is_empty() {
        output.push(LlmMessage {
            role: LlmMessageRole::Assistant,
            parts: text_parts,
            name: None,
        });
    }

    let mut normalized = LlmResponse::new(output);
    normalized.response_id = response
        .get("id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    normalized.model = response
        .get("model")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    normalized.tool_calls = tool_calls;
    normalized.structured_output = structured_output;
    normalized.finish_reason = response
        .get("stop_reason")
        .and_then(Value::as_str)
        .map(|reason| match reason {
            "end_turn" | "stop_sequence" => LlmFinishReason::Stop,
            "max_tokens" => LlmFinishReason::Length,
            "tool_use" => LlmFinishReason::ToolCalls,
            other => LlmFinishReason::Other(other.to_string()),
        });
    normalized.usage = response.get("usage").and_then(|usage| {
        Some(LlmUsage {
            input_tokens: usage.get("input_tokens")?.as_u64()?,
            output_tokens: usage.get("output_tokens")?.as_u64()?,
            total_tokens: usage.get("input_tokens")?.as_u64()?
                + usage.get("output_tokens")?.as_u64()?,
        })
    });
    normalized.conversation = Some(LlmConversationState {
        conversation_id: None,
        response_id: normalized.response_id.clone(),
        provider_state: None,
    });
    normalized.metadata = response.clone();
    Ok(normalized)
}

pub(crate) fn map_http_error(status: u16, body: &str) -> LlmError {
    let parsed = serde_json::from_str::<Value>(body).ok();
    let message = parsed
        .as_ref()
        .and_then(|value| value.get("error"))
        .and_then(|value| value.get("message"))
        .and_then(Value::as_str)
        .unwrap_or(body)
        .to_string();
    let prefixed = AnthropicError::api(message).to_string();

    match status {
        400 => LlmError::new(LlmErrorKind::InvalidRequest, prefixed),
        401 | 403 => LlmError::new(LlmErrorKind::Auth, prefixed),
        404 => LlmError::new(LlmErrorKind::UnsupportedModel, prefixed),
        408 => LlmError::new(LlmErrorKind::Timeout, prefixed).retryable(true),
        429 => LlmError::new(LlmErrorKind::RateLimited, prefixed).retryable(true),
        500..=599 => LlmError::new(LlmErrorKind::Provider, prefixed).retryable(true),
        _ => LlmError::new(LlmErrorKind::Provider, prefixed),
    }
}
