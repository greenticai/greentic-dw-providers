use greentic_dw_llm::{
    LlmContentPart, LlmError, LlmFinishReason, LlmMessage, LlmMessageRole, LlmRequest, LlmResponse,
    LlmResult, LlmToolCall, LlmToolChoice, LlmToolSpec, LlmUsage,
};
use serde_json::Value;

/// Normalized Bedrock request consumed by the transport.
#[derive(Clone, Debug, PartialEq)]
pub struct BedrockConverseRequest {
    /// System instructions collapsed into plain text blocks.
    pub system: Vec<String>,
    /// Ordered conversation messages.
    pub messages: Vec<BedrockMessage>,
    /// Declared tools for the request.
    pub tools: Vec<LlmToolSpec>,
    /// Tool calling policy.
    pub tool_choice: LlmToolChoice,
    /// Optional output token limit.
    pub max_output_tokens: Option<u32>,
    /// Optional temperature override.
    pub temperature: Option<f32>,
    /// Whether the caller requested streaming.
    pub stream: bool,
}

/// Normalized Bedrock message.
#[derive(Clone, Debug, PartialEq)]
pub struct BedrockMessage {
    /// Message role.
    pub role: BedrockMessageRole,
    /// Ordered content blocks.
    pub content: Vec<BedrockContentPart>,
}

/// Role supported by Bedrock Converse.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BedrockMessageRole {
    /// End-user or tool-result input.
    User,
    /// Assistant or model output.
    Assistant,
}

/// Content blocks carried into or out of Bedrock Converse.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum BedrockContentPart {
    /// Plain text content.
    Text(String),
    /// Tool use emitted by the model.
    ToolUse {
        /// Provider tool-use id.
        id: String,
        /// Tool name.
        name: String,
        /// Structured tool arguments.
        input: Value,
    },
    /// Tool result fed back into the model.
    ToolResult {
        /// Tool-use id being completed.
        tool_use_id: String,
        /// Structured result payload.
        result: Value,
        /// Whether the result is an error.
        is_error: bool,
    },
}

/// Normalized Bedrock Converse response produced by the transport.
#[derive(Clone, Debug, PartialEq)]
pub struct BedrockConverseResponse {
    /// Returned assistant message.
    pub message: BedrockMessage,
    /// Token usage when the backend reports it.
    pub usage: Option<LlmUsage>,
    /// Provider stop reason string.
    pub stop_reason: Option<String>,
}

pub(crate) fn build_converse_request(request: &LlmRequest) -> LlmResult<BedrockConverseRequest> {
    let mut system = Vec::new();
    let mut messages = Vec::new();

    for message in &request.messages {
        if message.role == LlmMessageRole::System {
            for part in &message.parts {
                match part {
                    LlmContentPart::Text { text } => system.push(text.clone()),
                    LlmContentPart::Json { value } => system.push(value.to_string()),
                    _ => {
                        return Err(LlmError::invalid_request(
                            "system messages only support text and json content for Bedrock",
                        ));
                    }
                }
            }
            continue;
        }

        let role = match message.role {
            LlmMessageRole::User | LlmMessageRole::Tool => BedrockMessageRole::User,
            LlmMessageRole::Assistant => BedrockMessageRole::Assistant,
            LlmMessageRole::System => unreachable!(),
        };
        messages.push(BedrockMessage {
            role,
            content: map_message_parts(&message.parts)?,
        });
    }

    Ok(BedrockConverseRequest {
        system,
        messages,
        tools: request.tools.clone(),
        tool_choice: request.tool_choice.clone(),
        max_output_tokens: request.max_output_tokens,
        temperature: request.temperature,
        stream: request.stream,
    })
}

fn map_message_parts(parts: &[LlmContentPart]) -> LlmResult<Vec<BedrockContentPart>> {
    parts
        .iter()
        .map(|part| match part {
            LlmContentPart::Text { text } => Ok(BedrockContentPart::Text(text.clone())),
            LlmContentPart::Json { value } => Ok(BedrockContentPart::Text(value.to_string())),
            LlmContentPart::ToolResult {
                tool_call_id,
                result,
                is_error,
                ..
            } => Ok(BedrockContentPart::ToolResult {
                tool_use_id: tool_call_id.clone(),
                result: result.clone(),
                is_error: *is_error,
            }),
            LlmContentPart::ImageUrl { .. } => Err(LlmError::invalid_request(
                "multimodal Bedrock request mapping is not enabled in this backend yet",
            )),
        })
        .collect()
}

pub(crate) fn parse_converse_response(response: BedrockConverseResponse) -> LlmResponse {
    let mut output = Vec::new();
    let mut tool_calls = Vec::new();
    let mut assistant_parts = Vec::new();

    for part in response.message.content {
        match part {
            BedrockContentPart::Text(text) => assistant_parts.push(LlmContentPart::Text { text }),
            BedrockContentPart::ToolUse { id, name, input } => {
                tool_calls.push(LlmToolCall {
                    id,
                    name,
                    arguments: input,
                });
            }
            BedrockContentPart::ToolResult { .. } => {}
        }
    }

    if !assistant_parts.is_empty() {
        output.push(LlmMessage {
            role: LlmMessageRole::Assistant,
            parts: assistant_parts,
            name: None,
        });
    }

    let mut normalized = LlmResponse::new(output);
    normalized.tool_calls = tool_calls;
    normalized.usage = response.usage;
    normalized.finish_reason = response.stop_reason.map(map_stop_reason);
    normalized
}

fn map_stop_reason(reason: String) -> LlmFinishReason {
    match reason.as_str() {
        "end_turn" => LlmFinishReason::Stop,
        "tool_use" => LlmFinishReason::ToolCalls,
        "max_tokens" => LlmFinishReason::Length,
        "content_filtered" => LlmFinishReason::ContentFilter,
        _ => LlmFinishReason::Other(reason),
    }
}
