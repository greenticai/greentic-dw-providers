use std::collections::HashMap;
use std::time::Duration;

use aws_config::BehaviorVersion;
use aws_credential_types::{Credentials, provider::SharedCredentialsProvider};
use aws_sdk_bedrockruntime::{
    Client,
    types::{
        AnyToolChoice, AutoToolChoice, ContentBlock, ConversationRole, ConverseOutput,
        InferenceConfiguration, Message, SystemContentBlock, Tool, ToolChoice, ToolConfiguration,
        ToolInputSchema, ToolResultBlock, ToolResultContentBlock, ToolResultStatus,
        ToolSpecification, ToolUseBlock,
    },
};
use aws_smithy_runtime_api::client::result::SdkError;
use aws_smithy_types::Document;
use aws_types::{SdkConfig, region::Region};
use greentic_dw_llm::{LlmError, LlmErrorKind, LlmResult, LlmToolChoice};
use serde_json::{Map, Number, Value};
use tokio::runtime::Builder as RuntimeBuilder;

use crate::config::{BedrockAuthMode, BedrockConfig};
use crate::mapping::{
    BedrockContentPart, BedrockConverseRequest, BedrockConverseResponse, BedrockMessage,
    BedrockMessageRole,
};

/// Transport abstraction used to execute Bedrock Converse calls.
pub trait BedrockTransport: Send + Sync {
    /// Executes a standard Converse request.
    fn converse(
        &self,
        config: &BedrockConfig,
        model_id: &str,
        request: &BedrockConverseRequest,
    ) -> LlmResult<BedrockConverseResponse>;

    /// Executes a text-oriented ConverseStream request.
    fn converse_stream(
        &self,
        config: &BedrockConfig,
        model_id: &str,
        request: &BedrockConverseRequest,
    ) -> LlmResult<BedrockConverseResponse>;
}

/// Default transport backed by the official AWS SDK for Bedrock Runtime.
#[derive(Default)]
pub struct AwsSdkBedrockTransport;

impl BedrockTransport for AwsSdkBedrockTransport {
    fn converse(
        &self,
        config: &BedrockConfig,
        model_id: &str,
        request: &BedrockConverseRequest,
    ) -> LlmResult<BedrockConverseResponse> {
        run_with_timeout(config.timeout_ms, async {
            let client = build_client(config).await?;
            let payload = build_sdk_payload(request)?;
            let mut fluent = client.converse().model_id(model_id.to_string());
            if !payload.messages.is_empty() {
                fluent = fluent.set_messages(Some(payload.messages));
            }
            if !payload.system.is_empty() {
                fluent = fluent.set_system(Some(payload.system));
            }
            if let Some(inference) = payload.inference_config {
                fluent = fluent.inference_config(inference);
            }
            if let Some(tool_config) = payload.tool_config {
                fluent = fluent.tool_config(tool_config);
            }

            let output = fluent
                .send()
                .await
                .map_err(|err| map_sdk_error("Converse", err))?;
            parse_converse_output(output)
        })
    }

    fn converse_stream(
        &self,
        config: &BedrockConfig,
        model_id: &str,
        request: &BedrockConverseRequest,
    ) -> LlmResult<BedrockConverseResponse> {
        run_with_timeout(config.timeout_ms, async {
            let client = build_client(config).await?;
            let payload = build_sdk_payload(request)?;
            let mut fluent = client.converse_stream().model_id(model_id.to_string());
            if !payload.messages.is_empty() {
                fluent = fluent.set_messages(Some(payload.messages));
            }
            if !payload.system.is_empty() {
                fluent = fluent.set_system(Some(payload.system));
            }
            if let Some(inference) = payload.inference_config {
                fluent = fluent.inference_config(inference);
            }
            if let Some(tool_config) = payload.tool_config {
                fluent = fluent.tool_config(tool_config);
            }

            let output = fluent
                .send()
                .await
                .map_err(|err| map_sdk_error("ConverseStream", err))?;
            parse_converse_stream_output(output).await
        })
    }
}

struct SdkPayload {
    messages: Vec<Message>,
    system: Vec<SystemContentBlock>,
    inference_config: Option<InferenceConfiguration>,
    tool_config: Option<ToolConfiguration>,
}

fn run_with_timeout<F, T>(timeout_ms: u64, future: F) -> LlmResult<T>
where
    F: std::future::Future<Output = LlmResult<T>>,
{
    let runtime = RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| {
            LlmError::new(
                LlmErrorKind::Internal,
                format!("failed to build tokio runtime for Bedrock transport: {err}"),
            )
        })?;

    runtime.block_on(async {
        tokio::time::timeout(Duration::from_millis(timeout_ms), future)
            .await
            .map_err(|_| {
                LlmError::new(
                    LlmErrorKind::Timeout,
                    format!("Bedrock request timed out after {timeout_ms}ms"),
                )
            })?
    })
}

async fn build_client(config: &BedrockConfig) -> LlmResult<Client> {
    let region = Region::new(config.region.clone());
    let mut loader = aws_config::defaults(BehaviorVersion::latest()).region(region);

    match &config.auth_mode {
        BedrockAuthMode::DefaultChain => {}
        BedrockAuthMode::Profile { profile_name } => {
            loader = loader.profile_name(profile_name.clone());
        }
        BedrockAuthMode::StaticKeys {
            access_key_id_secret,
            secret_access_key_secret,
            session_token_secret,
        } => {
            let credentials = Credentials::new(
                access_key_id_secret.clone(),
                secret_access_key_secret.clone(),
                session_token_secret.clone(),
                None,
                "GreenticBedrockStaticKeys",
            );
            loader = loader.credentials_provider(SharedCredentialsProvider::new(credentials));
        }
    }

    let sdk_config: SdkConfig = loader.load().await;
    Ok(Client::new(&sdk_config))
}

fn build_sdk_payload(request: &BedrockConverseRequest) -> LlmResult<SdkPayload> {
    let messages = request
        .messages
        .iter()
        .map(build_sdk_message)
        .collect::<LlmResult<Vec<_>>>()?;
    let system = request
        .system
        .iter()
        .cloned()
        .map(SystemContentBlock::Text)
        .collect::<Vec<_>>();
    let inference_config = build_inference_config(request);
    let tool_config = build_tool_config(request)?;

    Ok(SdkPayload {
        messages,
        system,
        inference_config,
        tool_config,
    })
}

fn build_sdk_message(message: &BedrockMessage) -> LlmResult<Message> {
    let role = match message.role {
        BedrockMessageRole::User => ConversationRole::User,
        BedrockMessageRole::Assistant => ConversationRole::Assistant,
    };
    let content = message
        .content
        .iter()
        .map(build_sdk_content_block)
        .collect::<LlmResult<Vec<_>>>()?;

    Message::builder()
        .role(role)
        .set_content(Some(content))
        .build()
        .map_err(build_error)
}

fn build_sdk_content_block(part: &BedrockContentPart) -> LlmResult<ContentBlock> {
    match part {
        BedrockContentPart::Text(text) => Ok(ContentBlock::Text(text.clone())),
        BedrockContentPart::ToolUse { id, name, input } => {
            let block = ToolUseBlock::builder()
                .tool_use_id(id.clone())
                .name(name.clone())
                .input(value_to_document(input)?)
                .build()
                .map_err(build_error)?;
            Ok(ContentBlock::ToolUse(block))
        }
        BedrockContentPart::ToolResult {
            tool_use_id,
            result,
            is_error,
        } => {
            let status = if *is_error {
                ToolResultStatus::Error
            } else {
                ToolResultStatus::Success
            };
            let content = match result {
                Value::String(text) => vec![ToolResultContentBlock::Text(text.clone())],
                other => vec![ToolResultContentBlock::Json(value_to_document(other)?)],
            };
            let block = ToolResultBlock::builder()
                .tool_use_id(tool_use_id.clone())
                .set_content(Some(content))
                .status(status)
                .build()
                .map_err(build_error)?;
            Ok(ContentBlock::ToolResult(block))
        }
    }
}

fn build_inference_config(request: &BedrockConverseRequest) -> Option<InferenceConfiguration> {
    if request.max_output_tokens.is_none() && request.temperature.is_none() {
        return None;
    }

    let mut builder = InferenceConfiguration::builder();
    if let Some(max_output_tokens) = request.max_output_tokens {
        builder = builder.max_tokens(max_output_tokens as i32);
    }
    if let Some(temperature) = request.temperature {
        builder = builder.temperature(temperature);
    }
    Some(builder.build())
}

fn build_tool_config(request: &BedrockConverseRequest) -> LlmResult<Option<ToolConfiguration>> {
    if request.tools.is_empty() {
        return Ok(None);
    }

    let tools = request
        .tools
        .iter()
        .map(|tool| {
            let mut spec = ToolSpecification::builder()
                .name(tool.name.clone())
                .input_schema(ToolInputSchema::Json(value_to_document(
                    &tool.input_schema,
                )?));
            if let Some(description) = &tool.description {
                spec = spec.description(description.clone());
            }
            Ok(Tool::ToolSpec(spec.build().map_err(build_error)?))
        })
        .collect::<LlmResult<Vec<_>>>()?;

    let tool_choice = match &request.tool_choice {
        LlmToolChoice::Auto => Some(ToolChoice::Auto(AutoToolChoice::builder().build())),
        LlmToolChoice::Required => Some(ToolChoice::Any(AnyToolChoice::builder().build())),
        LlmToolChoice::None => None,
        LlmToolChoice::Tool(name) => Some(ToolChoice::Tool(
            aws_sdk_bedrockruntime::types::SpecificToolChoice::builder()
                .name(name.clone())
                .build()
                .map_err(build_error)?,
        )),
    };

    ToolConfiguration::builder()
        .set_tools(Some(tools))
        .set_tool_choice(tool_choice)
        .build()
        .map(Some)
        .map_err(build_error)
}

fn parse_converse_output(
    output: aws_sdk_bedrockruntime::operation::converse::ConverseOutput,
) -> LlmResult<BedrockConverseResponse> {
    let message = match output.output {
        Some(ConverseOutput::Message(message)) => parse_sdk_message(message)?,
        Some(_) | None => BedrockMessage {
            role: BedrockMessageRole::Assistant,
            content: Vec::new(),
        },
    };

    Ok(BedrockConverseResponse {
        message,
        usage: output.usage.as_ref().map(convert_usage),
        stop_reason: Some(output.stop_reason.to_string()),
    })
}

async fn parse_converse_stream_output(
    output: aws_sdk_bedrockruntime::operation::converse_stream::ConverseStreamOutput,
) -> LlmResult<BedrockConverseResponse> {
    let mut stream = output.stream;
    let mut role = BedrockMessageRole::Assistant;
    let mut text_by_index: HashMap<i32, String> = HashMap::new();
    let mut tool_use_start_by_index: HashMap<i32, (String, String)> = HashMap::new();
    let mut tool_use_delta_by_index: HashMap<i32, String> = HashMap::new();
    let mut usage = None;
    let mut stop_reason = None;

    while let Some(event) = stream
        .recv()
        .await
        .map_err(|err| map_sdk_error("ConverseStream event", err))?
    {
        match event {
            aws_sdk_bedrockruntime::types::ConverseStreamOutput::MessageStart(start) => {
                role = match start.role {
                    ConversationRole::User => BedrockMessageRole::User,
                    ConversationRole::Assistant => BedrockMessageRole::Assistant,
                    _ => BedrockMessageRole::Assistant,
                };
            }
            aws_sdk_bedrockruntime::types::ConverseStreamOutput::ContentBlockStart(start) => {
                if let Some(aws_sdk_bedrockruntime::types::ContentBlockStart::ToolUse(tool_use)) =
                    start.start
                {
                    tool_use_start_by_index.insert(
                        start.content_block_index,
                        (tool_use.tool_use_id, tool_use.name),
                    );
                }
            }
            aws_sdk_bedrockruntime::types::ConverseStreamOutput::ContentBlockDelta(delta) => {
                if let Some(block_delta) = delta.delta {
                    match block_delta {
                        aws_sdk_bedrockruntime::types::ContentBlockDelta::Text(text) => {
                            text_by_index
                                .entry(delta.content_block_index)
                                .or_default()
                                .push_str(&text);
                        }
                        aws_sdk_bedrockruntime::types::ContentBlockDelta::ToolUse(tool_use) => {
                            tool_use_delta_by_index
                                .entry(delta.content_block_index)
                                .or_default()
                                .push_str(tool_use.input());
                        }
                        _ => {}
                    }
                }
            }
            aws_sdk_bedrockruntime::types::ConverseStreamOutput::MessageStop(stop) => {
                stop_reason = Some(stop.stop_reason.to_string());
            }
            aws_sdk_bedrockruntime::types::ConverseStreamOutput::Metadata(metadata) => {
                usage = metadata.usage.as_ref().map(convert_usage);
            }
            _ => {}
        }
    }

    let mut ordered_indices = text_by_index
        .keys()
        .copied()
        .chain(tool_use_start_by_index.keys().copied())
        .collect::<Vec<_>>();
    ordered_indices.sort_unstable();
    ordered_indices.dedup();

    let mut content = Vec::new();
    for index in ordered_indices {
        if let Some(text) = text_by_index.remove(&index) {
            content.push(BedrockContentPart::Text(text));
        }
        if let Some((id, name)) = tool_use_start_by_index.remove(&index) {
            let input = tool_use_delta_by_index.remove(&index).unwrap_or_default();
            let value = if input.trim().is_empty() {
                Value::Object(Map::new())
            } else {
                serde_json::from_str(&input).map_err(|err| {
                    LlmError::new(
                        LlmErrorKind::Provider,
                        format!("ConverseStream tool-use delta was not valid JSON: {err}"),
                    )
                })?
            };
            content.push(BedrockContentPart::ToolUse {
                id,
                name,
                input: value,
            });
        }
    }

    Ok(BedrockConverseResponse {
        message: BedrockMessage { role, content },
        usage,
        stop_reason,
    })
}

fn parse_sdk_message(message: Message) -> LlmResult<BedrockMessage> {
    let role = match message.role {
        ConversationRole::User => BedrockMessageRole::User,
        ConversationRole::Assistant => BedrockMessageRole::Assistant,
        _ => BedrockMessageRole::Assistant,
    };
    let content = message
        .content
        .into_iter()
        .filter_map(parse_sdk_content_block)
        .collect::<LlmResult<Vec<_>>>()?;

    Ok(BedrockMessage { role, content })
}

fn parse_sdk_content_block(block: ContentBlock) -> Option<LlmResult<BedrockContentPart>> {
    match block {
        ContentBlock::Text(text) => Some(Ok(BedrockContentPart::Text(text))),
        ContentBlock::ToolUse(tool_use) => {
            Some(
                document_to_value(tool_use.input).map(|input| BedrockContentPart::ToolUse {
                    id: tool_use.tool_use_id,
                    name: tool_use.name,
                    input,
                }),
            )
        }
        ContentBlock::ToolResult(tool_result) => {
            let tool_use_id = tool_result.tool_use_id.clone();
            Some(parse_tool_result(tool_result).map(|(result, is_error)| {
                BedrockContentPart::ToolResult {
                    tool_use_id,
                    result,
                    is_error,
                }
            }))
        }
        _ => None,
    }
}

fn parse_tool_result(tool_result: ToolResultBlock) -> LlmResult<(Value, bool)> {
    let result = tool_result
        .content
        .into_iter()
        .next()
        .map(|content| match content {
            ToolResultContentBlock::Text(text) => Ok(Value::String(text)),
            ToolResultContentBlock::Json(document) => document_to_value(document),
            _ => Ok(Value::Null),
        })
        .transpose()?
        .unwrap_or(Value::Null);

    let is_error = matches!(tool_result.status, Some(ToolResultStatus::Error));
    Ok((result, is_error))
}

fn convert_usage(usage: &aws_sdk_bedrockruntime::types::TokenUsage) -> greentic_dw_llm::LlmUsage {
    greentic_dw_llm::LlmUsage::new(
        usage.input_tokens.max(0) as u64,
        usage.output_tokens.max(0) as u64,
    )
}

fn value_to_document(value: &Value) -> LlmResult<Document> {
    Ok(match value {
        Value::Null => Document::Null,
        Value::Bool(boolean) => Document::Bool(*boolean),
        Value::Number(number) => number_to_document(number)?,
        Value::String(text) => Document::String(text.clone()),
        Value::Array(values) => {
            let docs = values
                .iter()
                .map(value_to_document)
                .collect::<LlmResult<Vec<_>>>()?;
            Document::Array(docs)
        }
        Value::Object(values) => {
            let docs = values
                .iter()
                .map(|(key, value)| value_to_document(value).map(|value| (key.clone(), value)))
                .collect::<LlmResult<HashMap<_, _>>>()?;
            Document::Object(docs)
        }
    })
}

fn number_to_document(number: &Number) -> LlmResult<Document> {
    if let Some(value) = number.as_u64() {
        return Ok(Document::from(value));
    }
    if let Some(value) = number.as_i64() {
        return Ok(Document::from(value));
    }
    if let Some(value) = number.as_f64() {
        return Ok(Document::from(value));
    }

    Err(LlmError::new(
        LlmErrorKind::Internal,
        format!("unsupported JSON number for Bedrock document conversion: {number}"),
    ))
}

fn document_to_value(document: Document) -> LlmResult<Value> {
    Ok(match document {
        Document::Null => Value::Null,
        Document::Bool(boolean) => Value::Bool(boolean),
        Document::String(text) => Value::String(text),
        Document::Number(number) => {
            let json_number = match number {
                aws_smithy_types::Number::PosInt(value) => Number::from(value),
                aws_smithy_types::Number::NegInt(value) => Number::from(value),
                aws_smithy_types::Number::Float(value) => {
                    Number::from_f64(value).ok_or_else(|| {
                        LlmError::new(
                            LlmErrorKind::Internal,
                            format!("Bedrock returned a non-finite float value: {value}"),
                        )
                    })?
                }
            };
            Value::Number(json_number)
        }
        Document::Array(values) => Value::Array(
            values
                .into_iter()
                .map(document_to_value)
                .collect::<LlmResult<Vec<_>>>()?,
        ),
        Document::Object(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| document_to_value(value).map(|value| (key, value)))
                .collect::<LlmResult<Map<_, _>>>()?,
        ),
    })
}

fn build_error(err: aws_smithy_types::error::operation::BuildError) -> LlmError {
    LlmError::new(
        LlmErrorKind::InvalidRequest,
        format!("failed to build Bedrock SDK request: {err}"),
    )
}

fn map_sdk_error<E, R>(operation: &str, err: SdkError<E, R>) -> LlmError
where
    E: std::error::Error + Send + Sync + 'static,
    R: std::fmt::Debug,
{
    let (kind, retryable) = match &err {
        SdkError::DispatchFailure(_) => (LlmErrorKind::Network, true),
        SdkError::TimeoutError(_) => (LlmErrorKind::Timeout, true),
        SdkError::ServiceError(service_err) => {
            let text = service_err.err().to_string().to_ascii_lowercase();
            if text.contains("throttl") || text.contains("quota") {
                (LlmErrorKind::RateLimited, true)
            } else if text.contains("access denied") || text.contains("unauthorized") {
                (LlmErrorKind::Auth, false)
            } else if text.contains("validation") {
                (LlmErrorKind::InvalidRequest, false)
            } else {
                (LlmErrorKind::Provider, false)
            }
        }
        _ => (LlmErrorKind::Provider, false),
    };

    LlmError::new(kind, format!("Bedrock {operation} failed: {err}")).retryable(retryable)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        BedrockContentPart, BedrockConverseRequest, BedrockMessage, BedrockMessageRole,
        build_error, build_inference_config, build_sdk_message, build_tool_config,
        document_to_value, parse_sdk_message, parse_tool_result, run_with_timeout,
        value_to_document,
    };
    use aws_sdk_bedrockruntime::types::{
        ContentBlock, ConversationRole, Message, ToolResultBlock, ToolResultContentBlock,
        ToolResultStatus, ToolUseBlock,
    };
    use aws_smithy_types::{Document, Number};
    use greentic_dw_llm::{LlmToolChoice, LlmToolSpec};
    use serde_json::json;
    use std::future;

    #[test]
    fn serde_json_value_round_trips_through_document() {
        let value = json!({
            "message": "hello",
            "count": 3,
            "ratio": 0.5,
            "ok": true,
            "items": ["a", {"nested": null}]
        });

        let document = value_to_document(&value).expect("document");
        let round_trip = document_to_value(document).expect("value");
        assert_eq!(round_trip, value);
    }

    #[test]
    fn smithy_document_numbers_convert_back_to_json_numbers() {
        let float = document_to_value(Document::Number(Number::Float(1.25))).expect("float");
        let negative = document_to_value(Document::Number(Number::NegInt(-4))).expect("negative");
        let positive = document_to_value(Document::Number(Number::PosInt(7))).expect("positive");

        assert_eq!(float, json!(1.25));
        assert_eq!(negative, json!(-4));
        assert_eq!(positive, json!(7));
    }

    #[test]
    fn build_inference_config_is_empty_when_request_has_no_limits() {
        let request = BedrockConverseRequest {
            system: Vec::new(),
            messages: Vec::new(),
            max_output_tokens: None,
            temperature: None,
            tools: Vec::new(),
            tool_choice: LlmToolChoice::None,
            stream: false,
        };
        assert!(build_inference_config(&request).is_none());
    }

    #[test]
    fn build_inference_config_maps_limits_and_temperature() {
        let request = BedrockConverseRequest {
            system: Vec::new(),
            messages: Vec::new(),
            max_output_tokens: Some(128),
            temperature: Some(0.4),
            tools: Vec::new(),
            tool_choice: LlmToolChoice::None,
            stream: false,
        };
        assert!(build_inference_config(&request).is_some());
    }

    #[test]
    fn build_tool_config_maps_required_choice_and_schema() {
        let request = BedrockConverseRequest {
            system: Vec::new(),
            messages: Vec::new(),
            max_output_tokens: None,
            temperature: None,
            tools: vec![
                LlmToolSpec::new("lookup", json!({"type":"object"}))
                    .expect("tool")
                    .with_description("Lookup data"),
            ],
            tool_choice: LlmToolChoice::Required,
            stream: false,
        };
        let config = build_tool_config(&request).expect("tool config");
        assert!(config.is_some());
    }

    #[test]
    fn build_sdk_message_maps_tool_use_and_tool_result_parts() {
        let message = BedrockMessage {
            role: BedrockMessageRole::Assistant,
            content: vec![
                BedrockContentPart::Text("hello".to_string()),
                BedrockContentPart::ToolUse {
                    id: "call_1".to_string(),
                    name: "lookup".to_string(),
                    input: json!({"id":1}),
                },
                BedrockContentPart::ToolResult {
                    tool_use_id: "call_1".to_string(),
                    result: json!({"ok":true}),
                    is_error: false,
                },
            ],
        };
        let sdk = build_sdk_message(&message).expect("sdk message");
        assert_eq!(sdk.role, ConversationRole::Assistant);
        assert_eq!(sdk.content.len(), 3);
    }

    #[test]
    fn parse_sdk_message_maps_tool_use_and_tool_result_parts() {
        let tool_use = ToolUseBlock::builder()
            .tool_use_id("call_1")
            .name("lookup")
            .input(Document::Object(
                [("id".to_string(), Document::from(1_u64))]
                    .into_iter()
                    .collect(),
            ))
            .build()
            .expect("tool use");
        let tool_result = ToolResultBlock::builder()
            .tool_use_id("call_1")
            .content(ToolResultContentBlock::Text("done".to_string()))
            .status(ToolResultStatus::Success)
            .build()
            .expect("tool result");
        let sdk = Message::builder()
            .role(ConversationRole::Assistant)
            .content(ContentBlock::Text("hello".to_string()))
            .content(ContentBlock::ToolUse(tool_use))
            .content(ContentBlock::ToolResult(tool_result))
            .build()
            .expect("message");

        let parsed = parse_sdk_message(sdk).expect("parsed");
        assert_eq!(parsed.role, BedrockMessageRole::Assistant);
        assert_eq!(parsed.content.len(), 3);
    }

    #[test]
    fn parse_tool_result_tracks_error_status() {
        let tool_result = ToolResultBlock::builder()
            .tool_use_id("call_1")
            .content(ToolResultContentBlock::Json(Document::from(7_i64)))
            .status(ToolResultStatus::Error)
            .build()
            .expect("tool result");
        let (result, is_error) = parse_tool_result(tool_result).expect("parsed");
        assert_eq!(result, json!(7));
        assert!(is_error);
    }

    #[test]
    fn run_with_timeout_reports_timeout() {
        let err = run_with_timeout(1, future::pending::<greentic_dw_llm::LlmResult<()>>())
            .expect_err("timeout");
        assert_eq!(err.kind, greentic_dw_llm::LlmErrorKind::Timeout);
    }

    #[test]
    fn build_error_maps_sdk_build_failures_to_invalid_request() {
        let err = Message::builder()
            .build()
            .expect_err("missing role and content");
        let mapped = build_error(err);
        assert_eq!(mapped.kind, greentic_dw_llm::LlmErrorKind::InvalidRequest);
        assert!(
            mapped
                .message
                .contains("failed to build Bedrock SDK request")
        );
    }
}
