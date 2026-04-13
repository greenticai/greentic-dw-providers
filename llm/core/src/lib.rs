#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Shared normalized LLM provider contract.
//!
//! The goal of this crate is to define a provider-neutral request and response
//! model for LLM backends without baking any one vendor's transport semantics
//! into the core abstraction.

use greentic_cap_types::CapabilityProfile;
use greentic_types::TenantCtx;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error as StdError;
use std::fmt;

/// Canonical runtime capability URI for the LLM family.
pub const LLM_CAPABILITY_URI: &str = "cap://dw.llm";

/// Canonical pack capability identifier for the LLM family.
pub const LLM_PACK_CAPABILITY_ID: &str = "greentic.cap.llm";

/// Small summary of the current contract state for the LLM family.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmFamilyMetadata {
    /// Canonical family slug.
    pub family: &'static str,
    /// Shared runtime capability URI.
    pub capability_uri: &'static str,
    /// Shared pack capability id.
    pub pack_capability_id: &'static str,
    /// Whether the normalized provider contract is available.
    pub contract_ready: bool,
}

/// Role associated with a message in the normalized transcript.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmMessageRole {
    /// System-level instruction or guardrail.
    System,
    /// End-user input.
    User,
    /// Assistant output.
    Assistant,
    /// Tool or function result.
    Tool,
}

/// Multi-part content carried inside a normalized LLM message.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LlmContentPart {
    /// Plain text content.
    Text {
        /// Text payload.
        text: String,
    },
    /// Structured JSON payload.
    Json {
        /// JSON value payload.
        value: Value,
    },
    /// Image reference by URL or remote location.
    ImageUrl {
        /// Image location.
        url: String,
        /// Optional media type.
        media_type: Option<String>,
    },
    /// Tool output fed back into the model.
    ToolResult {
        /// Tool call identifier.
        tool_call_id: String,
        /// Stable tool name.
        tool_name: String,
        /// Structured tool result.
        result: Value,
        /// Whether the tool result represents an error.
        is_error: bool,
    },
}

impl LlmContentPart {
    /// Creates a text content part.
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    fn validate(&self) -> LlmResult<()> {
        match self {
            Self::Text { text } if text.trim().is_empty() => {
                Err(LlmError::invalid_request("text content must not be empty"))
            }
            Self::ImageUrl { url, .. } if url.trim().is_empty() => {
                Err(LlmError::invalid_request("image url must not be empty"))
            }
            Self::ToolResult {
                tool_call_id,
                tool_name,
                ..
            } if tool_call_id.trim().is_empty() || tool_name.trim().is_empty() => Err(
                LlmError::invalid_request("tool result ids and names must not be empty"),
            ),
            _ => Ok(()),
        }
    }

    fn is_non_text_input(&self) -> bool {
        !matches!(self, Self::Text { .. } | Self::ToolResult { .. })
    }
}

/// Normalized message shape used by LLM requests and responses.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LlmMessage {
    /// Message role.
    pub role: LlmMessageRole,
    /// Ordered content parts.
    pub parts: Vec<LlmContentPart>,
    /// Optional participant name.
    pub name: Option<String>,
}

impl LlmMessage {
    /// Creates a new message from a role and ordered parts.
    pub fn new(role: LlmMessageRole, parts: Vec<LlmContentPart>) -> LlmResult<Self> {
        if parts.is_empty() {
            return Err(LlmError::invalid_request(
                "llm message must contain at least one content part",
            ));
        }
        for part in &parts {
            part.validate()?;
        }
        Ok(Self {
            role,
            parts,
            name: None,
        })
    }

    /// Creates a single-part text message.
    pub fn text(role: LlmMessageRole, text: impl Into<String>) -> LlmResult<Self> {
        Self::new(role, vec![LlmContentPart::text(text)])
    }

    /// Sets an optional message name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    fn validate(&self) -> LlmResult<()> {
        if self.parts.is_empty() {
            return Err(LlmError::invalid_request(
                "llm message must contain at least one content part",
            ));
        }
        for part in &self.parts {
            part.validate()?;
        }
        Ok(())
    }
}

/// Tool or function schema exposed to the model.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LlmToolSpec {
    /// Stable tool name.
    pub name: String,
    /// Optional human-readable description.
    pub description: Option<String>,
    /// Input schema the model should target.
    pub input_schema: Value,
}

impl LlmToolSpec {
    /// Creates a new tool specification.
    pub fn new(name: impl Into<String>, input_schema: Value) -> LlmResult<Self> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(LlmError::invalid_request("tool name must not be empty"));
        }
        Ok(Self {
            name,
            description: None,
            input_schema,
        })
    }

    /// Adds a human-readable description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Tool selection policy for a request.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmToolChoice {
    /// Let the backend choose automatically.
    Auto,
    /// Do not allow tool use for this request.
    #[default]
    None,
    /// Require at least one tool call if supported.
    Required,
    /// Force a specific tool name.
    Tool(String),
}

/// Structured-output target carried with a request.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LlmStructuredOutputSpec {
    /// Stable schema name.
    pub name: String,
    /// Optional human-readable description.
    pub description: Option<String>,
    /// JSON schema describing the expected output.
    pub schema: Value,
    /// Whether the backend should enforce strict schema conformance.
    pub strict: bool,
}

impl LlmStructuredOutputSpec {
    /// Creates a new structured-output target.
    pub fn new(name: impl Into<String>, schema: Value) -> LlmResult<Self> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(LlmError::invalid_request(
                "structured output name must not be empty",
            ));
        }
        Ok(Self {
            name,
            description: None,
            schema,
            strict: true,
        })
    }
}

/// Opaque conversation or provider-managed state returned by stateful backends.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmConversationState {
    /// Stable conversation identifier when the backend exposes one.
    pub conversation_id: Option<String>,
    /// Stable response identifier or turn identifier when the backend exposes one.
    pub response_id: Option<String>,
    /// Provider-defined opaque state token.
    pub provider_state: Option<String>,
}

impl LlmConversationState {
    /// Creates an empty conversation state handle.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            conversation_id: None,
            response_id: None,
            provider_state: None,
        }
    }
}

impl Default for LlmConversationState {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalized request sent to an LLM provider.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LlmRequest {
    /// Stable request identifier.
    pub request_id: String,
    /// Optional per-request model override.
    pub model: Option<String>,
    /// Ordered prompt transcript.
    pub messages: Vec<LlmMessage>,
    /// Optional tool declarations.
    pub tools: Vec<LlmToolSpec>,
    /// Tool selection policy.
    pub tool_choice: LlmToolChoice,
    /// Optional structured-output target.
    pub structured_output: Option<LlmStructuredOutputSpec>,
    /// Optional output-token cap.
    pub max_output_tokens: Option<u32>,
    /// Optional temperature override.
    pub temperature: Option<f32>,
    /// Whether the caller prefers a streaming response.
    pub stream: bool,
    /// Optional provider-managed conversation state.
    pub conversation: Option<LlmConversationState>,
    /// Optional normalized metadata bag.
    pub metadata: Value,
}

impl LlmRequest {
    /// Creates a new request from an id and prompt messages.
    pub fn new(request_id: impl Into<String>, messages: Vec<LlmMessage>) -> LlmResult<Self> {
        let request_id = request_id.into();
        if request_id.trim().is_empty() {
            return Err(LlmError::invalid_request(
                "llm request id must not be empty",
            ));
        }
        if messages.is_empty() {
            return Err(LlmError::invalid_request(
                "llm request must contain at least one message",
            ));
        }
        let request = Self {
            request_id,
            model: None,
            messages,
            tools: Vec::new(),
            tool_choice: LlmToolChoice::None,
            structured_output: None,
            max_output_tokens: None,
            temperature: None,
            stream: false,
            conversation: None,
            metadata: Value::Null,
        };
        request.validate()?;
        Ok(request)
    }

    /// Sets the model override.
    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Adds a tool declaration.
    #[must_use]
    pub fn with_tool(mut self, tool: LlmToolSpec) -> Self {
        self.tools.push(tool);
        self
    }

    /// Sets the tool choice.
    #[must_use]
    pub fn with_tool_choice(mut self, tool_choice: LlmToolChoice) -> Self {
        self.tool_choice = tool_choice;
        self
    }

    /// Sets a structured-output target.
    #[must_use]
    pub fn with_structured_output(mut self, structured_output: LlmStructuredOutputSpec) -> Self {
        self.structured_output = Some(structured_output);
        self
    }

    /// Marks the request as streaming.
    #[must_use]
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    /// Attaches provider-managed conversation state.
    #[must_use]
    pub fn with_conversation(mut self, conversation: LlmConversationState) -> Self {
        self.conversation = Some(conversation);
        self
    }

    /// Attaches normalized metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Validates the request for basic structural correctness.
    pub fn validate(&self) -> LlmResult<()> {
        if self.request_id.trim().is_empty() {
            return Err(LlmError::invalid_request(
                "llm request id must not be empty",
            ));
        }
        if self.messages.is_empty() {
            return Err(LlmError::invalid_request(
                "llm request must contain at least one message",
            ));
        }
        for message in &self.messages {
            message.validate()?;
        }
        if matches!(
            self.tool_choice,
            LlmToolChoice::Required | LlmToolChoice::Tool(_)
        ) && self.tools.is_empty()
        {
            return Err(LlmError::invalid_request(
                "tool choice requires at least one declared tool",
            ));
        }
        if let LlmToolChoice::Tool(tool_name) = &self.tool_choice
            && !self.tools.iter().any(|tool| &tool.name == tool_name)
        {
            return Err(LlmError::invalid_request(
                "forced tool choice must reference a declared tool",
            ));
        }
        Ok(())
    }

    /// Returns whether the request needs chat semantics beyond plain text completion.
    #[must_use]
    pub fn requires_chat_features(&self) -> bool {
        self.messages.len() > 1
            || self
                .messages
                .iter()
                .any(|message| message.role != LlmMessageRole::User)
    }

    /// Returns whether the request contains non-textual model input.
    #[must_use]
    pub fn requires_multimodal_input(&self) -> bool {
        self.messages
            .iter()
            .any(|message| message.parts.iter().any(LlmContentPart::is_non_text_input))
    }
}

/// Model or backend tool call emitted by a provider.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LlmToolCall {
    /// Stable tool call identifier.
    pub id: String,
    /// Tool name selected by the model.
    pub name: String,
    /// Structured arguments payload.
    pub arguments: Value,
}

/// Token usage summary returned by a provider.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmUsage {
    /// Input or prompt token count.
    pub input_tokens: u64,
    /// Output or completion token count.
    pub output_tokens: u64,
    /// Total token count when the backend exposes it.
    pub total_tokens: u64,
}

impl LlmUsage {
    /// Creates a usage summary from input and output counts.
    #[must_use]
    pub const fn new(input_tokens: u64, output_tokens: u64) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
        }
    }
}

/// Normalized completion finish reason.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmFinishReason {
    /// Natural stop.
    Stop,
    /// Token or length limit reached.
    Length,
    /// Model emitted a tool call.
    ToolCalls,
    /// Backend content filtering stopped the response.
    ContentFilter,
    /// Provider-specific finish reason.
    Other(String),
}

/// Normalized response returned by an LLM provider.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LlmResponse {
    /// Optional provider response identifier.
    pub response_id: Option<String>,
    /// Model name resolved by the backend.
    pub model: Option<String>,
    /// Normalized output messages.
    pub output: Vec<LlmMessage>,
    /// Tool calls emitted by the model.
    pub tool_calls: Vec<LlmToolCall>,
    /// Structured output payload when requested and supported.
    pub structured_output: Option<Value>,
    /// Optional token usage.
    pub usage: Option<LlmUsage>,
    /// Finish reason if exposed by the backend.
    pub finish_reason: Option<LlmFinishReason>,
    /// Updated provider-managed conversation state.
    pub conversation: Option<LlmConversationState>,
    /// Provider-specific normalized metadata.
    pub metadata: Value,
}

impl LlmResponse {
    /// Creates an empty response shell.
    #[must_use]
    pub fn new(output: Vec<LlmMessage>) -> Self {
        Self {
            response_id: None,
            model: None,
            output,
            tool_calls: Vec::new(),
            structured_output: None,
            usage: None,
            finish_reason: None,
            conversation: None,
            metadata: Value::Null,
        }
    }

    /// Collects the plain text emitted by assistant messages.
    #[must_use]
    pub fn text_output(&self) -> String {
        self.output
            .iter()
            .flat_map(|message| message.parts.iter())
            .filter_map(|part| match part {
                LlmContentPart::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

/// Provider capability flags exposed by the normalized contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmProviderFeatures {
    /// Supports chat-style multi-message requests.
    pub chat: bool,
    /// Supports schema-constrained structured outputs.
    pub structured_outputs: bool,
    /// Supports tool or function calling.
    pub tool_calling: bool,
    /// Supports streaming responses.
    pub streaming: bool,
    /// Supports multimodal request inputs.
    pub multimodal_input: bool,
    /// Supports provider-managed conversation state.
    pub stateful_conversation: bool,
    /// Can run in local or self-hosted environments.
    pub local_self_hosted: bool,
    /// Supports enterprise auth patterns such as IAM, SSO, or workload identity.
    pub enterprise_auth: bool,
}

impl LlmProviderFeatures {
    /// Creates a new feature set.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        chat: bool,
        structured_outputs: bool,
        tool_calling: bool,
        streaming: bool,
        multimodal_input: bool,
        stateful_conversation: bool,
        local_self_hosted: bool,
        enterprise_auth: bool,
    ) -> Self {
        Self {
            chat,
            structured_outputs,
            tool_calling,
            streaming,
            multimodal_input,
            stateful_conversation,
            local_self_hosted,
            enterprise_auth,
        }
    }

    fn enabled_feature_names(&self) -> Vec<&'static str> {
        let mut features = Vec::new();
        if self.chat {
            features.push("chat");
        }
        if self.structured_outputs {
            features.push("structured_outputs");
        }
        if self.tool_calling {
            features.push("tool_calling");
        }
        if self.streaming {
            features.push("streaming");
        }
        if self.multimodal_input {
            features.push("multimodal_input");
        }
        if self.stateful_conversation {
            features.push("stateful_conversation");
        }
        if self.local_self_hosted {
            features.push("local_self_hosted");
        }
        if self.enterprise_auth {
            features.push("enterprise_auth");
        }
        features
    }

    /// Returns the enabled operation set implied by the feature flags.
    #[must_use]
    pub fn operations(&self) -> Vec<String> {
        let mut operations = vec!["llm.generate".to_string()];
        if self.chat {
            operations.push("llm.chat".to_string());
        }
        if self.streaming {
            operations.push("llm.stream".to_string());
        }
        if self.tool_calling {
            operations.push("llm.tools".to_string());
        }
        if self.structured_outputs {
            operations.push("llm.structured".to_string());
        }
        if self.multimodal_input {
            operations.push("llm.multimodal".to_string());
        }
        if self.stateful_conversation {
            operations.push("llm.stateful".to_string());
        }
        operations
    }

    /// Builds a capability-profile entry summarizing the enabled flags.
    #[must_use]
    pub fn as_capability_profile(&self, profile_id: impl Into<String>) -> CapabilityProfile {
        let mut profile = CapabilityProfile::new(profile_id);
        let enabled = self.enabled_feature_names();
        profile.description = Some(if enabled.is_empty() {
            "LLM feature profile with no optional features enabled".to_string()
        } else {
            format!("LLM feature profile: {}", enabled.join(", "))
        });
        profile
    }

    /// Validates whether the feature set can satisfy a normalized request.
    pub fn validate_request(&self, request: &LlmRequest) -> LlmResult<()> {
        request.validate()?;

        if request.requires_chat_features() && !self.chat {
            return Err(LlmError::unsupported_feature(
                "chat",
                "provider does not support chat-style requests",
            ));
        }
        if request.stream && !self.streaming {
            return Err(LlmError::unsupported_feature(
                "streaming",
                "provider does not support streaming",
            ));
        }
        if request.structured_output.is_some() && !self.structured_outputs {
            return Err(LlmError::unsupported_feature(
                "structured_outputs",
                "provider does not support structured outputs",
            ));
        }
        if (!request.tools.is_empty() || !matches!(request.tool_choice, LlmToolChoice::None))
            && !self.tool_calling
        {
            return Err(LlmError::unsupported_feature(
                "tool_calling",
                "provider does not support tool calling",
            ));
        }
        if request.conversation.is_some() && !self.stateful_conversation {
            return Err(LlmError::unsupported_feature(
                "stateful_conversation",
                "provider does not support stateful conversations",
            ));
        }
        if request.requires_multimodal_input() && !self.multimodal_input {
            return Err(LlmError::unsupported_feature(
                "multimodal_input",
                "provider does not support multimodal input",
            ));
        }
        Ok(())
    }
}

/// High-level LLM error category.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LlmErrorKind {
    /// Input or request validation failure.
    InvalidRequest,
    /// Requested feature is unsupported by the provider or model.
    UnsupportedFeature,
    /// Requested model is unsupported or unavailable.
    UnsupportedModel,
    /// Authentication or authorization failure.
    Auth,
    /// Timeout while waiting for the provider.
    Timeout,
    /// Network or transport failure.
    Network,
    /// Rate limiting or quota failure.
    RateLimited,
    /// Provider-specific failure that does not fit a narrower category.
    Provider,
    /// Internal or unexpected failure.
    Internal,
}

/// Normalized LLM error returned by the contract.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmError {
    /// Error category.
    pub kind: LlmErrorKind,
    /// Human-readable error message.
    pub message: String,
    /// Optional feature name associated with an unsupported-feature error.
    pub feature: Option<String>,
    /// Whether the failure is retryable.
    pub retryable: bool,
}

impl LlmError {
    /// Creates a new error.
    #[must_use]
    pub fn new(kind: LlmErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            feature: None,
            retryable: false,
        }
    }

    /// Creates an invalid-request error.
    #[must_use]
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(LlmErrorKind::InvalidRequest, message)
    }

    /// Creates an unsupported-feature error.
    #[must_use]
    pub fn unsupported_feature(feature: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: LlmErrorKind::UnsupportedFeature,
            message: message.into(),
            feature: Some(feature.into()),
            retryable: false,
        }
    }

    /// Marks an error as retryable.
    #[must_use]
    pub fn retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.feature {
            Some(feature) => write!(f, "{} ({feature})", self.message),
            None => f.write_str(&self.message),
        }
    }
}

impl StdError for LlmError {}

/// Result type used by the normalized LLM contract.
pub type LlmResult<T> = Result<T, LlmError>;

/// Contract implemented by LLM backends.
pub trait LlmProvider: Send + Sync {
    /// Returns the provider's declared feature set.
    fn features(&self) -> &LlmProviderFeatures;

    /// Executes a normalized request and returns a normalized response.
    fn generate(&self, tenant: &TenantCtx, request: LlmRequest) -> LlmResult<LlmResponse>;

    /// Validates a request against the provider's declared capabilities.
    fn validate_request(&self, request: &LlmRequest) -> LlmResult<()> {
        self.features().validate_request(request)
    }
}

/// Returns the current LLM family metadata.
#[must_use]
pub const fn llm_family_metadata() -> LlmFamilyMetadata {
    LlmFamilyMetadata {
        family: "llm",
        capability_uri: LLM_CAPABILITY_URI,
        pack_capability_id: LLM_PACK_CAPABILITY_ID,
        contract_ready: true,
    }
}

/// Sample requests and feature sets used by provider conformance tests.
pub mod fixtures {
    use super::{
        LlmContentPart, LlmConversationState, LlmMessage, LlmMessageRole, LlmProviderFeatures,
        LlmRequest, LlmResult, LlmStructuredOutputSpec, LlmToolChoice, LlmToolSpec,
    };
    use serde_json::{Value, json};

    /// Returns a feature set that supports plain text and chat but no optional extras.
    #[must_use]
    pub const fn basic_features() -> LlmProviderFeatures {
        LlmProviderFeatures::new(true, false, false, false, false, false, false, false)
    }

    /// Returns a feature set that supports chat, tools, structured outputs, streaming, and statefulness.
    #[must_use]
    pub const fn rich_features() -> LlmProviderFeatures {
        LlmProviderFeatures::new(true, true, true, true, false, true, false, true)
    }

    /// Returns a plain text request fixture.
    pub fn text_request() -> LlmResult<LlmRequest> {
        LlmRequest::new(
            "req-text",
            vec![LlmMessage::text(LlmMessageRole::User, "Write a greeting")?],
        )
    }

    /// Returns a structured-output request fixture.
    pub fn structured_output_request() -> LlmResult<LlmRequest> {
        Ok(
            text_request()?.with_structured_output(LlmStructuredOutputSpec::new(
                "greeting",
                json!({
                    "type": "object",
                    "properties": {
                        "message": { "type": "string" }
                    },
                    "required": ["message"]
                }),
            )?),
        )
    }

    /// Returns a tool-call request fixture.
    pub fn tool_request() -> LlmResult<LlmRequest> {
        Ok(text_request()?
            .with_tool(LlmToolSpec::new(
                "lookup_weather",
                json!({
                    "type": "object",
                    "properties": {
                        "city": { "type": "string" }
                    },
                    "required": ["city"]
                }),
            )?)
            .with_tool_choice(LlmToolChoice::Auto))
    }

    /// Returns a request fixture that requires multimodal support.
    pub fn multimodal_request() -> LlmResult<LlmRequest> {
        LlmRequest::new(
            "req-multimodal",
            vec![LlmMessage::new(
                LlmMessageRole::User,
                vec![
                    LlmContentPart::text("Describe this image"),
                    LlmContentPart::ImageUrl {
                        url: "https://example.com/image.png".to_string(),
                        media_type: Some("image/png".to_string()),
                    },
                ],
            )?],
        )
    }

    /// Returns a request fixture that requires stateful conversation support.
    pub fn stateful_request() -> LlmResult<LlmRequest> {
        Ok(text_request()?.with_conversation(LlmConversationState {
            conversation_id: Some("conv-1".to_string()),
            response_id: Some("resp-1".to_string()),
            provider_state: Some("opaque-state".to_string()),
        }))
    }

    /// Returns a generic metadata payload used by tests.
    #[must_use]
    pub fn sample_metadata() -> Value {
        json!({"source": "llm/core-fixture"})
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::fixtures::{
        basic_features, multimodal_request, rich_features, stateful_request,
        structured_output_request, text_request, tool_request,
    };
    use super::{
        LLM_CAPABILITY_URI, LLM_PACK_CAPABILITY_ID, LlmContentPart, LlmFinishReason, LlmMessage,
        LlmMessageRole, LlmProviderFeatures, LlmResponse, LlmToolCall, llm_family_metadata,
    };
    use serde_json::json;

    #[test]
    fn family_metadata_matches_contract_constants() {
        let metadata = llm_family_metadata();
        assert_eq!(metadata.family, "llm");
        assert_eq!(metadata.capability_uri, LLM_CAPABILITY_URI);
        assert_eq!(metadata.pack_capability_id, LLM_PACK_CAPABILITY_ID);
        assert!(metadata.contract_ready);
    }

    #[test]
    fn request_validation_rejects_missing_messages() {
        let err = super::LlmRequest::new("req-empty", Vec::new()).unwrap_err();
        assert_eq!(err.kind, super::LlmErrorKind::InvalidRequest);
    }

    #[test]
    fn request_validation_rejects_forced_tool_without_declaration() {
        let err = text_request()
            .expect("fixture")
            .with_tool_choice(super::LlmToolChoice::Tool("lookup_weather".to_string()))
            .validate()
            .unwrap_err();
        assert_eq!(err.kind, super::LlmErrorKind::InvalidRequest);
    }

    #[test]
    fn basic_features_reject_optional_request_features() {
        let features = basic_features();
        let err = features
            .validate_request(&structured_output_request().expect("fixture"))
            .unwrap_err();
        assert_eq!(err.kind, super::LlmErrorKind::UnsupportedFeature);
        assert_eq!(err.feature.as_deref(), Some("structured_outputs"));

        let err = features
            .validate_request(&tool_request().expect("fixture"))
            .unwrap_err();
        assert_eq!(err.feature.as_deref(), Some("tool_calling"));

        let err = features
            .validate_request(&stateful_request().expect("fixture"))
            .unwrap_err();
        assert_eq!(err.feature.as_deref(), Some("stateful_conversation"));
    }

    #[test]
    fn multimodal_requests_require_multimodal_feature_flag() {
        let err = basic_features()
            .validate_request(&multimodal_request().expect("fixture"))
            .unwrap_err();
        assert_eq!(err.feature.as_deref(), Some("multimodal_input"));

        let features =
            LlmProviderFeatures::new(true, false, false, false, true, false, true, false);
        assert!(
            features
                .validate_request(&multimodal_request().expect("fixture"))
                .is_ok()
        );
    }

    #[test]
    fn rich_features_accept_supported_request_shapes() {
        let features = rich_features();
        assert!(
            features
                .validate_request(&text_request().expect("fixture"))
                .is_ok()
        );
        assert!(
            features
                .validate_request(&structured_output_request().expect("fixture"))
                .is_ok()
        );
        assert!(
            features
                .validate_request(&tool_request().expect("fixture").with_stream(true))
                .is_ok()
        );
        assert!(
            features
                .validate_request(&stateful_request().expect("fixture"))
                .is_ok()
        );
    }

    #[test]
    fn feature_profiles_and_operations_are_derived_from_flags() {
        let features = rich_features();
        let profile = features.as_capability_profile("llm.provider.features");
        assert_eq!(profile.id, "llm.provider.features");
        assert!(
            profile
                .description
                .as_deref()
                .expect("profile description should be present")
                .contains("tool_calling")
        );
        assert!(
            features
                .operations()
                .iter()
                .any(|op| op == "llm.structured")
        );
        assert!(features.operations().iter().any(|op| op == "llm.stateful"));
    }

    #[test]
    fn response_text_output_collects_text_parts() {
        let response = LlmResponse {
            response_id: Some("resp-1".to_string()),
            model: Some("model-x".to_string()),
            output: vec![
                LlmMessage::new(
                    LlmMessageRole::Assistant,
                    vec![
                        LlmContentPart::text("Hello"),
                        LlmContentPart::text(", world"),
                    ],
                )
                .expect("response message should be valid"),
            ],
            tool_calls: vec![LlmToolCall {
                id: "tool-1".to_string(),
                name: "lookup_weather".to_string(),
                arguments: json!({"city": "Amsterdam"}),
            }],
            structured_output: Some(json!({"message": "Hello, world"})),
            usage: Some(super::LlmUsage::new(10, 5)),
            finish_reason: Some(LlmFinishReason::Stop),
            conversation: None,
            metadata: json!({"source": "test"}),
        };

        assert_eq!(response.text_output(), "Hello, world");
    }
}
