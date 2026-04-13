use greentic_dw_llm::{
    LlmContentPart, LlmConversationState, LlmError, LlmErrorKind, LlmFinishReason, LlmMessage,
    LlmMessageRole, LlmRequest, LlmResponse, LlmResult, LlmToolCall, LlmToolChoice, LlmUsage,
};
use serde_json::{Map, Value, json};

use crate::config::OpenAiConfig;

pub(crate) fn build_responses_request(
    config: &OpenAiConfig,
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
        "input".to_string(),
        Value::Array(build_input_items(request)?),
    );
    body.insert("stream".to_string(), Value::Bool(request.stream));

    if let Some(max_output_tokens) = request.max_output_tokens {
        body.insert(
            "max_output_tokens".to_string(),
            Value::Number(max_output_tokens.into()),
        );
    }
    if let Some(temperature) = request.temperature {
        body.insert("temperature".to_string(), json!(temperature));
    }
    if let Some(reasoning_profile) = config.reasoning_profile {
        body.insert(
            "reasoning".to_string(),
            json!({ "effort": reasoning_profile.as_str() }),
        );
    }
    if let Some(conversation) = &request.conversation
        && let Some(response_id) = &conversation.response_id
    {
        body.insert(
            "previous_response_id".to_string(),
            Value::String(response_id.clone()),
        );
    }
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
                            "name": tool.name,
                            "description": tool.description,
                            "parameters": tool.input_schema,
                            "strict": true,
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
    } else if !request.tools.is_empty() {
        body.insert("tool_choice".to_string(), Value::String("none".to_string()));
    }
    if let Some(structured_output) = &request.structured_output {
        body.insert(
            "text".to_string(),
            json!({
                "format": {
                    "type": "json_schema",
                    "name": structured_output.name,
                    "description": structured_output.description,
                    "schema": structured_output.schema,
                    "strict": structured_output.strict,
                }
            }),
        );
    }
    if request.metadata != Value::Null {
        body.insert("metadata".to_string(), request.metadata.clone());
    }

    Ok(Value::Object(body))
}

fn map_tool_choice(choice: &LlmToolChoice) -> Value {
    match choice {
        LlmToolChoice::Auto => Value::String("auto".to_string()),
        LlmToolChoice::None => Value::String("none".to_string()),
        LlmToolChoice::Required => Value::String("required".to_string()),
        LlmToolChoice::Tool(name) => json!({
            "type": "function",
            "name": name,
        }),
    }
}

fn build_input_items(request: &LlmRequest) -> LlmResult<Vec<Value>> {
    let mut items = Vec::new();
    for message in &request.messages {
        match message.role {
            LlmMessageRole::Tool => {
                for part in &message.parts {
                    match part {
                        LlmContentPart::ToolResult {
                            tool_call_id,
                            result,
                            ..
                        } => {
                            items.push(json!({
                                "type": "function_call_output",
                                "call_id": tool_call_id,
                                "output": result.to_string(),
                            }));
                        }
                        _ => {
                            return Err(LlmError::invalid_request(
                                "tool role messages must contain tool results",
                            ));
                        }
                    }
                }
            }
            _ => {
                items.push(json!({
                    "type": "message",
                    "role": role_name(message.role),
                    "content": map_message_parts(&message.parts)?,
                }));
            }
        }
    }
    Ok(items)
}

fn role_name(role: LlmMessageRole) -> &'static str {
    match role {
        LlmMessageRole::System => "system",
        LlmMessageRole::User => "user",
        LlmMessageRole::Assistant => "assistant",
        LlmMessageRole::Tool => "tool",
    }
}

fn map_message_parts(parts: &[LlmContentPart]) -> LlmResult<Vec<Value>> {
    parts
        .iter()
        .map(|part| match part {
            LlmContentPart::Text { text } => Ok(json!({
                "type": "input_text",
                "text": text,
            })),
            LlmContentPart::Json { value } => Ok(json!({
                "type": "input_text",
                "text": value.to_string(),
            })),
            LlmContentPart::ImageUrl { url, .. } => Ok(json!({
                "type": "input_image",
                "image_url": url,
                "detail": "auto",
            })),
            LlmContentPart::ToolResult { .. } => Err(LlmError::invalid_request(
                "tool results must be carried in tool-role messages",
            )),
        })
        .collect()
}

pub(crate) fn parse_responses_response(
    response: &Value,
    request: &LlmRequest,
) -> LlmResult<LlmResponse> {
    let output_items = response
        .get("output")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut output_messages = Vec::new();
    let mut tool_calls = Vec::new();

    for item in output_items {
        match item.get("type").and_then(Value::as_str) {
            Some("message") => {
                let role = match item
                    .get("role")
                    .and_then(Value::as_str)
                    .unwrap_or("assistant")
                {
                    "system" => LlmMessageRole::System,
                    "user" => LlmMessageRole::User,
                    "tool" => LlmMessageRole::Tool,
                    _ => LlmMessageRole::Assistant,
                };
                let parts = item
                    .get("content")
                    .and_then(Value::as_array)
                    .map(|content| {
                        content
                            .iter()
                            .filter_map(parse_output_part)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                if !parts.is_empty() {
                    output_messages.push(LlmMessage {
                        role,
                        parts,
                        name: None,
                    });
                }
            }
            Some("function_call") => {
                let arguments = item
                    .get("arguments")
                    .and_then(Value::as_str)
                    .and_then(|value| serde_json::from_str::<Value>(value).ok())
                    .unwrap_or(Value::Null);
                tool_calls.push(LlmToolCall {
                    id: item
                        .get("call_id")
                        .and_then(Value::as_str)
                        .or_else(|| item.get("id").and_then(Value::as_str))
                        .unwrap_or_default()
                        .to_string(),
                    name: item
                        .get("name")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                    arguments,
                });
            }
            _ => {}
        }
    }

    let usage = response.get("usage").and_then(parse_usage);
    let response_id = response
        .get("id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    let model = response
        .get("model")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);

    let mut normalized = LlmResponse::new(output_messages);
    normalized.response_id = response_id.clone();
    normalized.model = model;
    normalized.tool_calls = tool_calls;
    normalized.usage = usage;
    normalized.finish_reason = response
        .get("status")
        .and_then(Value::as_str)
        .map(parse_finish_reason);
    normalized.conversation = response_id.map(|response_id| LlmConversationState {
        conversation_id: None,
        response_id: Some(response_id),
        provider_state: None,
    });
    normalized.metadata = response.clone();

    if request.structured_output.is_some() {
        let text = normalized.text_output();
        if !text.is_empty() {
            normalized.structured_output = serde_json::from_str(&text).ok();
        }
    }

    Ok(normalized)
}

fn parse_output_part(value: &Value) -> Option<LlmContentPart> {
    match value.get("type").and_then(Value::as_str) {
        Some("output_text") => {
            value
                .get("text")
                .and_then(Value::as_str)
                .map(|text| LlmContentPart::Text {
                    text: text.to_string(),
                })
        }
        Some("input_text") => {
            value
                .get("text")
                .and_then(Value::as_str)
                .map(|text| LlmContentPart::Text {
                    text: text.to_string(),
                })
        }
        _ => None,
    }
}

fn parse_usage(value: &Value) -> Option<LlmUsage> {
    let input_tokens = value.get("input_tokens")?.as_u64()?;
    let output_tokens = value.get("output_tokens")?.as_u64()?;
    Some(LlmUsage {
        input_tokens,
        output_tokens,
        total_tokens: value
            .get("total_tokens")
            .and_then(Value::as_u64)
            .unwrap_or(input_tokens + output_tokens),
    })
}

fn parse_finish_reason(status: &str) -> LlmFinishReason {
    match status {
        "completed" => LlmFinishReason::Stop,
        "incomplete" => LlmFinishReason::Length,
        other => LlmFinishReason::Other(other.to_string()),
    }
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

    match status {
        400 => LlmError::new(LlmErrorKind::InvalidRequest, message),
        401 | 403 => LlmError::new(LlmErrorKind::Auth, message),
        404 | 422 => LlmError::new(LlmErrorKind::UnsupportedModel, message),
        408 => LlmError::new(LlmErrorKind::Timeout, message).retryable(true),
        429 => LlmError::new(LlmErrorKind::RateLimited, message).retryable(true),
        500..=599 => LlmError::new(LlmErrorKind::Provider, message).retryable(true),
        _ => LlmError::new(LlmErrorKind::Provider, message),
    }
}
