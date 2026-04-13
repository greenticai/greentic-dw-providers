use greentic_dw_llm::{
    LlmContentPart, LlmError, LlmErrorKind, LlmFinishReason, LlmMessage, LlmMessageRole,
    LlmRequest, LlmResponse, LlmResult, LlmToolCall, LlmToolChoice, LlmUsage,
};
use serde_json::{Map, Value, json};

use crate::config::GeminiConfig;

pub(crate) fn build_generate_content_request(
    config: &GeminiConfig,
    request: &LlmRequest,
) -> LlmResult<Value> {
    let mut body = Map::new();
    body.insert(
        "contents".to_string(),
        Value::Array(build_contents(request)?),
    );

    if let Some(system_instruction) = build_system_instruction(request)? {
        body.insert("systemInstruction".to_string(), system_instruction);
    }

    if !request.tools.is_empty() {
        body.insert(
            "tools".to_string(),
            Value::Array(vec![json!({
                "functionDeclarations": request.tools.iter().map(|tool| json!({
                    "name": tool.name,
                    "description": tool.description,
                    "parameters": tool.input_schema,
                })).collect::<Vec<_>>()
            })]),
        );

        let tool_config = match &request.tool_choice {
            LlmToolChoice::None => Some(json!({
                "functionCallingConfig": { "mode": "NONE" }
            })),
            LlmToolChoice::Auto => Some(json!({
                "functionCallingConfig": { "mode": "AUTO" }
            })),
            LlmToolChoice::Required => Some(json!({
                "functionCallingConfig": { "mode": "ANY" }
            })),
            LlmToolChoice::Tool(name) => Some(json!({
                "functionCallingConfig": {
                    "mode": "ANY",
                    "allowedFunctionNames": [name],
                }
            })),
        };
        if let Some(tool_config) = tool_config {
            body.insert("toolConfig".to_string(), tool_config);
        }
    }

    let mut generation_config = Map::new();
    if let Some(max_tokens) = request.max_output_tokens {
        generation_config.insert("maxOutputTokens".to_string(), json!(max_tokens));
    }
    if let Some(temperature) = request.temperature {
        generation_config.insert("temperature".to_string(), json!(temperature));
    }
    if let Some(structured_output) = &request.structured_output {
        generation_config.insert(
            "responseMimeType".to_string(),
            Value::String("application/json".to_string()),
        );
        generation_config.insert(
            "responseJsonSchema".to_string(),
            structured_output.schema.clone(),
        );
    }
    if !generation_config.is_empty() {
        body.insert(
            "generationConfig".to_string(),
            Value::Object(generation_config),
        );
    }

    if let Some(profile) = &config.safety_profile {
        body.insert(
            "labels".to_string(),
            json!({
                "safety_profile": profile,
            }),
        );
    }

    Ok(Value::Object(body))
}

fn build_system_instruction(request: &LlmRequest) -> LlmResult<Option<Value>> {
    let mut parts = Vec::new();
    for message in &request.messages {
        if message.role == LlmMessageRole::System {
            for part in &message.parts {
                match part {
                    LlmContentPart::Text { text } => parts.push(json!({ "text": text })),
                    LlmContentPart::Json { value } => {
                        parts.push(json!({ "text": value.to_string() }))
                    }
                    _ => {
                        return Err(LlmError::invalid_request(
                            "system messages only support text and json content for Gemini",
                        ));
                    }
                }
            }
        }
    }
    if parts.is_empty() {
        Ok(None)
    } else {
        Ok(Some(json!({ "parts": parts })))
    }
}

fn build_contents(request: &LlmRequest) -> LlmResult<Vec<Value>> {
    let mut contents = Vec::new();
    for message in &request.messages {
        if message.role == LlmMessageRole::System {
            continue;
        }
        let role = match message.role {
            LlmMessageRole::Assistant => "model",
            LlmMessageRole::User | LlmMessageRole::Tool => "user",
            LlmMessageRole::System => unreachable!(),
        };
        contents.push(json!({
            "role": role,
            "parts": map_message_parts(&message.parts)?,
        }));
    }
    Ok(contents)
}

fn map_message_parts(parts: &[LlmContentPart]) -> LlmResult<Vec<Value>> {
    parts
        .iter()
        .map(|part| match part {
            LlmContentPart::Text { text } => Ok(json!({ "text": text })),
            LlmContentPart::Json { value } => Ok(json!({ "text": value.to_string() })),
            LlmContentPart::ToolResult {
                tool_name, result, ..
            } => Ok(json!({
                "functionResponse": {
                    "name": tool_name,
                    "response": result,
                }
            })),
            LlmContentPart::ImageUrl { .. } => Err(LlmError::invalid_request(
                "Gemini multimodal URL inputs are not enabled in this backend yet",
            )),
        })
        .collect()
}

pub(crate) fn parse_generate_content_response(
    response: &Value,
    request: &LlmRequest,
) -> LlmResult<LlmResponse> {
    let candidate = response
        .get("candidates")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .ok_or_else(|| {
            LlmError::new(
                LlmErrorKind::Provider,
                "Gemini response did not include a candidate",
            )
        })?;

    let parts = candidate
        .get("content")
        .and_then(|content: &Value| content.get("parts"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    let mut assistant_parts = Vec::new();
    let mut text_buffer = String::new();
    let mut tool_calls = Vec::new();

    for (index, part) in parts.iter().enumerate() {
        if let Some(text) = part.get("text").and_then(Value::as_str) {
            text_buffer.push_str(text);
        }
        if let Some(function_call) = part.get("functionCall") {
            tool_calls.push(LlmToolCall {
                id: format!("gemini-call-{}", index + 1),
                name: function_call
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                arguments: function_call.get("args").cloned().unwrap_or(Value::Null),
            });
        }
    }

    let structured_output = if request.structured_output.is_some() && !text_buffer.trim().is_empty()
    {
        Some(serde_json::from_str(&text_buffer).map_err(|err| {
            LlmError::new(
                LlmErrorKind::Provider,
                format!("Gemini structured output response was not valid JSON: {err}"),
            )
        })?)
    } else {
        None
    };

    if structured_output.is_none() && !text_buffer.is_empty() {
        assistant_parts.push(LlmContentPart::Text { text: text_buffer });
    }

    let output = if assistant_parts.is_empty() {
        Vec::new()
    } else {
        vec![LlmMessage {
            role: LlmMessageRole::Assistant,
            parts: assistant_parts,
            name: None,
        }]
    };

    let mut normalized = LlmResponse::new(output);
    normalized.model = response
        .get("modelVersion")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned);
    normalized.tool_calls = tool_calls;
    normalized.structured_output = structured_output;
    normalized.finish_reason = candidate
        .get("finishReason")
        .and_then(Value::as_str)
        .map(map_finish_reason);
    normalized.usage = response.get("usageMetadata").map(map_usage);
    Ok(normalized)
}

fn map_finish_reason(reason: &str) -> LlmFinishReason {
    match reason {
        "STOP" | "FINISH_REASON_UNSPECIFIED" => LlmFinishReason::Stop,
        "MAX_TOKENS" => LlmFinishReason::Length,
        "SAFETY" => LlmFinishReason::ContentFilter,
        other => LlmFinishReason::Other(other.to_lowercase()),
    }
}

fn map_usage(value: &Value) -> LlmUsage {
    LlmUsage {
        input_tokens: value
            .get("promptTokenCount")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        output_tokens: value
            .get("candidatesTokenCount")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        total_tokens: value
            .get("totalTokenCount")
            .and_then(Value::as_u64)
            .unwrap_or(0),
    }
}

pub(crate) fn map_http_error(status: u16, body: &str) -> LlmError {
    let detail = extract_error_message(body).unwrap_or_else(|| body.trim().to_string());
    let message = format!("Gemini generateContent request failed: {detail}");
    match status {
        400 => LlmError::new(LlmErrorKind::InvalidRequest, message),
        401 | 403 => LlmError::new(LlmErrorKind::Auth, message),
        404 => LlmError::new(LlmErrorKind::UnsupportedModel, message),
        408 => LlmError::new(LlmErrorKind::Timeout, message).retryable(true),
        429 => LlmError::new(LlmErrorKind::RateLimited, message).retryable(true),
        500..=599 => LlmError::new(LlmErrorKind::Provider, message).retryable(true),
        _ => LlmError::new(LlmErrorKind::Provider, message),
    }
}

fn extract_error_message(body: &str) -> Option<String> {
    let parsed: Value = serde_json::from_str(body).ok()?;
    parsed
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}
