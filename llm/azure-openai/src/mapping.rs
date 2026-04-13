use greentic_dw_llm::{
    LlmContentPart, LlmConversationState, LlmError, LlmErrorKind, LlmFinishReason, LlmMessage,
    LlmMessageRole, LlmRequest, LlmResponse, LlmResult, LlmToolCall, LlmToolChoice, LlmUsage,
};
use serde_json::{Map, Value, json};

use crate::config::AzureOpenAiConfig;

pub(crate) fn build_request(config: &AzureOpenAiConfig, request: &LlmRequest) -> LlmResult<Value> {
    if config.use_responses_api {
        build_responses_request(config, request)
    } else {
        build_chat_completions_request(config, request)
    }
}

fn build_responses_request(config: &AzureOpenAiConfig, request: &LlmRequest) -> LlmResult<Value> {
    let mut body = Map::new();
    body.insert(
        "model".to_string(),
        Value::String(
            request
                .model
                .clone()
                .unwrap_or_else(|| config.deployment.clone()),
        ),
    );
    body.insert(
        "input".to_string(),
        Value::Array(build_responses_input(request)?),
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
    if config.allow_stateful_responses
        && let Some(conversation) = &request.conversation
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
            map_responses_tool_choice(&request.tool_choice),
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

fn build_chat_completions_request(
    config: &AzureOpenAiConfig,
    request: &LlmRequest,
) -> LlmResult<Value> {
    let mut body = Map::new();
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

    if let Some(max_output_tokens) = request.max_output_tokens {
        body.insert(
            "max_tokens".to_string(),
            Value::Number(max_output_tokens.into()),
        );
    }
    if let Some(temperature) = request.temperature {
        body.insert("temperature".to_string(), json!(temperature));
    }
    body.insert(
        "model".to_string(),
        Value::String(config.deployment.clone()),
    );

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
            map_chat_tool_choice(&request.tool_choice),
        );
    }
    if let Some(structured_output) = &request.structured_output {
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

fn build_responses_input(request: &LlmRequest) -> LlmResult<Vec<Value>> {
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
                        } => items.push(json!({
                            "type": "function_call_output",
                            "call_id": tool_call_id,
                            "output": result.to_string(),
                        })),
                        _ => {
                            return Err(LlmError::invalid_request(
                                "tool role messages must contain tool results",
                            ));
                        }
                    }
                }
            }
            _ => items.push(json!({
                "type": "message",
                "role": role_name(message.role),
                "content": message
                    .parts
                    .iter()
                    .map(map_responses_part)
                    .collect::<LlmResult<Vec<_>>>()?,
            })),
        }
    }
    Ok(items)
}

fn map_responses_part(part: &LlmContentPart) -> LlmResult<Value> {
    match part {
        LlmContentPart::Text { text } => Ok(json!({ "type": "input_text", "text": text })),
        LlmContentPart::Json { value } => {
            Ok(json!({ "type": "input_text", "text": value.to_string() }))
        }
        LlmContentPart::ImageUrl { url, .. } => {
            Ok(json!({ "type": "input_image", "image_url": url, "detail": "auto" }))
        }
        LlmContentPart::ToolResult { .. } => Err(LlmError::invalid_request(
            "tool results must be carried in tool-role messages",
        )),
    }
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
                "tool results are not supported in chat message content mapping",
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

fn map_responses_tool_choice(choice: &LlmToolChoice) -> Value {
    match choice {
        LlmToolChoice::Auto => json!("auto"),
        LlmToolChoice::None => json!("none"),
        LlmToolChoice::Required => json!("required"),
        LlmToolChoice::Tool(name) => json!({ "type": "function", "name": name }),
    }
}

fn map_chat_tool_choice(choice: &LlmToolChoice) -> Value {
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

pub(crate) fn parse_response(
    config: &AzureOpenAiConfig,
    value: &Value,
    request: &LlmRequest,
) -> LlmResult<LlmResponse> {
    if config.use_responses_api {
        parse_responses_response(value, request)
    } else {
        parse_chat_completions_response(value, request)
    }
}

fn parse_responses_response(value: &Value, request: &LlmRequest) -> LlmResult<LlmResponse> {
    let output_items = value
        .get("output")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut output = Vec::new();
    let mut tool_calls = Vec::new();

    for item in output_items {
        match item.get("type").and_then(Value::as_str) {
            Some("message") => {
                let parts = item
                    .get("content")
                    .and_then(Value::as_array)
                    .map(|content| {
                        content
                            .iter()
                            .filter_map(|part| match part.get("type").and_then(Value::as_str) {
                                Some("output_text") => part
                                    .get("text")
                                    .and_then(Value::as_str)
                                    .map(|text| LlmContentPart::Text {
                                        text: text.to_string(),
                                    }),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                if !parts.is_empty() {
                    output.push(LlmMessage {
                        role: LlmMessageRole::Assistant,
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

    let mut response = LlmResponse::new(output);
    response.tool_calls = tool_calls;
    response.response_id = value
        .get("id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    response.model = value
        .get("model")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    response.finish_reason = value
        .get("status")
        .and_then(Value::as_str)
        .map(parse_finish_reason);
    response.usage = parse_usage(value.get("usage"));
    response.conversation = response
        .response_id
        .clone()
        .map(|response_id| LlmConversationState {
            conversation_id: None,
            response_id: Some(response_id),
            provider_state: None,
        });
    response.metadata = value.clone();

    if request.structured_output.is_some() {
        let text = response.text_output();
        if !text.is_empty() {
            response.structured_output = serde_json::from_str(&text).ok();
        }
    }

    Ok(response)
}

fn parse_chat_completions_response(value: &Value, request: &LlmRequest) -> LlmResult<LlmResponse> {
    let choice = value
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|choices| choices.first())
        .ok_or_else(|| {
            LlmError::new(
                LlmErrorKind::Provider,
                "chat completions response missing choices",
            )
        })?;
    let message = choice.get("message").ok_or_else(|| {
        LlmError::new(
            LlmErrorKind::Provider,
            "chat completions response missing message",
        )
    })?;

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
                .and_then(|function| function.get("arguments"))
                .and_then(Value::as_str)
                .and_then(|value| serde_json::from_str::<Value>(value).ok())
                .unwrap_or(Value::Null);
            tool_calls.push(LlmToolCall {
                id: call
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                name: call
                    .get("function")
                    .and_then(|function| function.get("name"))
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                arguments,
            });
        }
    }

    let mut response = LlmResponse::new(output);
    response.tool_calls = tool_calls;
    response.model = value
        .get("model")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    response.finish_reason = choice
        .get("finish_reason")
        .and_then(Value::as_str)
        .map(parse_chat_finish_reason);
    response.usage = parse_usage(value.get("usage"));
    response.metadata = value.clone();

    if request.structured_output.is_some() {
        let text = response.text_output();
        if !text.is_empty() {
            response.structured_output = serde_json::from_str(&text).ok();
        }
    }

    Ok(response)
}

fn parse_usage(value: Option<&Value>) -> Option<LlmUsage> {
    let value = value?;
    let input_tokens = value
        .get("input_tokens")
        .or_else(|| value.get("prompt_tokens"))?
        .as_u64()?;
    let output_tokens = value
        .get("output_tokens")
        .or_else(|| value.get("completion_tokens"))?
        .as_u64()?;
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

fn parse_chat_finish_reason(finish_reason: &str) -> LlmFinishReason {
    match finish_reason {
        "stop" => LlmFinishReason::Stop,
        "length" => LlmFinishReason::Length,
        "tool_calls" => LlmFinishReason::ToolCalls,
        other => LlmFinishReason::Other(other.to_string()),
    }
}

pub(crate) fn map_http_error(status: u16, body: &str) -> LlmError {
    let parsed = serde_json::from_str::<Value>(body).ok();
    let message = parsed
        .as_ref()
        .and_then(|value| value.get("error"))
        .and_then(|value| value.get("message"))
        .or_else(|| parsed.as_ref().and_then(|value| value.get("message")))
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
