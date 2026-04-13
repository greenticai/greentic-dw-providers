use semver::Version;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    CapabilityDeclaration, CapabilityIdError, CapabilityProfile, ProviderCategory,
    ProviderDeclSpec, capability_declaration, capability_provider_ref, provider_decl,
    provider_extension_inline, provider_manifest,
};
use greentic_dw_llm::LlmProviderFeatures;

use greentic_types::{
    CapabilitiesExtensionError, CborError, PackId, PackKind, PackManifest, PackSignatures,
    ProviderDecl, ProviderExtensionInline, ProviderManifest, encode_pack_manifest,
};

/// Canonical capability URI shared by the LLM provider family.
pub const LLM_CAPABILITY_URI: &str = greentic_dw_llm::LLM_CAPABILITY_URI;

/// Canonical pack capability identifier shared by the LLM provider family.
pub const LLM_PACK_CAPABILITY_ID: &str = greentic_dw_llm::LLM_PACK_CAPABILITY_ID;

/// Canonical provider slug for the native OpenAI backend.
pub const OPENAI_LLM_PROVIDER_NAME: &str = "openai";

/// Canonical provider slug for the Azure OpenAI backend.
pub const AZURE_OPENAI_LLM_PROVIDER_NAME: &str = "azure-openai";

/// Canonical provider slug for the Anthropic backend.
pub const ANTHROPIC_LLM_PROVIDER_NAME: &str = "anthropic";

/// Canonical provider slug for the Bedrock backend.
pub const BEDROCK_LLM_PROVIDER_NAME: &str = "bedrock";

/// Canonical provider slug for the generic OpenAI-compatible backend.
pub const OPENAI_COMPATIBLE_LLM_PROVIDER_NAME: &str = "openai-compatible";

/// Canonical provider slug for the Gemini backend.
pub const GEMINI_LLM_PROVIDER_NAME: &str = "gemini";

/// Canonical provider slug for the NVIDIA NIM backend.
pub const NVIDIA_NIM_LLM_PROVIDER_NAME: &str = "nvidia-nim";

/// Alias for the canonical LLM provider feature model defined in `llm/core`.
pub type LlmFeatureProfile = LlmProviderFeatures;

/// Typed wizard question kind shared across LLM providers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LlmWizardQuestionKind {
    /// Arbitrary string input.
    Text,
    /// Secret reference or secret-like token input.
    SecretRef,
    /// Integer-like numeric input.
    Number,
    /// Boolean toggle.
    Boolean,
    /// Closed selection from provider-owned options.
    Select {
        /// User-selectable options.
        options: Vec<LlmWizardQuestionOption>,
    },
}

/// A single selectable option for a wizard question.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmWizardQuestionOption {
    /// Stable machine-readable option value.
    pub value: String,
    /// I18n-ready label key for the option.
    pub label_key: String,
}

impl LlmWizardQuestionOption {
    /// Builds a wizard select option.
    #[must_use]
    pub fn new(value: impl Into<String>, label_key: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label_key: label_key.into(),
        }
    }
}

/// Visibility rule owned by a provider question.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "rule", rename_all = "snake_case")]
pub enum LlmWizardVisibility {
    /// Always show the question.
    Always,
    /// Show only when another answer equals a specific value.
    Equals {
        /// Source question key.
        key: String,
        /// Required value for visibility.
        value: String,
    },
}

/// Provider-owned wizard question descriptor shared across LLM backends.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmWizardQuestion {
    /// Stable question key.
    pub key: String,
    /// I18n-ready prompt key.
    pub prompt_key: String,
    /// Whether the answer is required.
    pub required: bool,
    /// Typed answer surface for the wizard.
    pub kind: LlmWizardQuestionKind,
    /// Optional default answer expressed as a string value.
    pub default_value: Option<String>,
    /// Optional visibility rule.
    pub visibility: LlmWizardVisibility,
}

impl LlmWizardQuestion {
    /// Creates a free-text question.
    #[must_use]
    pub fn text(key: impl Into<String>, prompt_key: impl Into<String>, required: bool) -> Self {
        Self::new(key, prompt_key, required, LlmWizardQuestionKind::Text)
    }

    /// Creates a secret reference question.
    #[must_use]
    pub fn secret_ref(
        key: impl Into<String>,
        prompt_key: impl Into<String>,
        required: bool,
    ) -> Self {
        Self::new(key, prompt_key, required, LlmWizardQuestionKind::SecretRef)
    }

    /// Creates a number question.
    #[must_use]
    pub fn number(key: impl Into<String>, prompt_key: impl Into<String>, required: bool) -> Self {
        Self::new(key, prompt_key, required, LlmWizardQuestionKind::Number)
    }

    /// Creates a boolean question.
    #[must_use]
    pub fn boolean(key: impl Into<String>, prompt_key: impl Into<String>, required: bool) -> Self {
        Self::new(key, prompt_key, required, LlmWizardQuestionKind::Boolean)
    }

    /// Creates a select question.
    #[must_use]
    pub fn select(
        key: impl Into<String>,
        prompt_key: impl Into<String>,
        required: bool,
        options: Vec<LlmWizardQuestionOption>,
    ) -> Self {
        Self::new(
            key,
            prompt_key,
            required,
            LlmWizardQuestionKind::Select { options },
        )
    }

    /// Assigns a default value to the question.
    #[must_use]
    pub fn with_default(mut self, default_value: impl Into<String>) -> Self {
        self.default_value = Some(default_value.into());
        self
    }

    /// Marks the question as visible only when another answer equals a specific value.
    #[must_use]
    pub fn visible_when_equals(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.visibility = LlmWizardVisibility::Equals {
            key: key.into(),
            value: value.into(),
        };
        self
    }

    fn new(
        key: impl Into<String>,
        prompt_key: impl Into<String>,
        required: bool,
        kind: LlmWizardQuestionKind,
    ) -> Self {
        Self {
            key: key.into(),
            prompt_key: prompt_key.into(),
            required,
            kind,
            default_value: None,
            visibility: LlmWizardVisibility::Always,
        }
    }
}

/// Provider-level wizard metadata block owned by an LLM backend.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LlmWizardProviderQa {
    /// Canonical provider slug.
    pub provider_name: String,
    /// I18n-ready provider label key.
    pub provider_label_key: String,
    /// Whether the provider should be suggested by default in generic UX.
    pub default_selected: bool,
    /// Compatibility or deployment tags exposed to the wizard.
    pub compatibility_tags: Vec<String>,
    /// Provider-owned follow-up questions.
    pub questions: Vec<LlmWizardQuestion>,
}

impl LlmWizardProviderQa {
    /// Builds provider-owned wizard metadata.
    #[must_use]
    pub fn new(
        provider_name: impl Into<String>,
        provider_label_key: impl Into<String>,
        default_selected: bool,
        compatibility_tags: Vec<String>,
        questions: Vec<LlmWizardQuestion>,
    ) -> Self {
        Self {
            provider_name: provider_name.into(),
            provider_label_key: provider_label_key.into(),
            default_selected,
            compatibility_tags,
            questions,
        }
    }
}

/// Returns the provider-owned wizard metadata for native OpenAI setup.
#[must_use]
pub fn openai_llm_wizard_qa() -> LlmWizardProviderQa {
    LlmWizardProviderQa::new(
        OPENAI_LLM_PROVIDER_NAME,
        "wizard.llm.provider.openai.label",
        true,
        vec![
            "native".to_string(),
            "saas".to_string(),
            "responses".to_string(),
        ],
        vec![
            LlmWizardQuestion::secret_ref(
                "api_key_secret",
                "wizard.llm.openai.api_key_secret",
                true,
            ),
            LlmWizardQuestion::text("model", "wizard.llm.shared.model", true),
            LlmWizardQuestion::text("base_url", "wizard.llm.shared.base_url", false),
            LlmWizardQuestion::text("organization", "wizard.llm.openai.organization", false),
            LlmWizardQuestion::text("project", "wizard.llm.openai.project", false),
            LlmWizardQuestion::number("timeout_ms", "wizard.llm.shared.timeout_ms", true)
                .with_default("60000"),
            LlmWizardQuestion::boolean("allow_tools", "wizard.llm.shared.allow_tools", true)
                .with_default("true"),
            LlmWizardQuestion::boolean(
                "allow_structured_outputs",
                "wizard.llm.shared.allow_structured_outputs",
                true,
            )
            .with_default("true"),
            LlmWizardQuestion::text(
                "reasoning_profile",
                "wizard.llm.openai.reasoning_profile",
                false,
            ),
        ],
    )
}

/// Returns the provider-owned wizard metadata for Azure OpenAI setup.
#[must_use]
pub fn azure_openai_llm_wizard_qa() -> LlmWizardProviderQa {
    LlmWizardProviderQa::new(
        AZURE_OPENAI_LLM_PROVIDER_NAME,
        "wizard.llm.provider.azure_openai.label",
        false,
        vec![
            "azure".to_string(),
            "enterprise".to_string(),
            "responses".to_string(),
        ],
        vec![
            LlmWizardQuestion::text("endpoint", "wizard.llm.azure_openai.endpoint", true),
            LlmWizardQuestion::text("deployment", "wizard.llm.azure_openai.deployment", true),
            LlmWizardQuestion::text("api_version", "wizard.llm.azure_openai.api_version", true)
                .with_default("v1"),
            LlmWizardQuestion::select(
                "auth_mode",
                "wizard.llm.azure_openai.auth_mode",
                true,
                vec![
                    LlmWizardQuestionOption::new(
                        "api_key",
                        "wizard.llm.azure_openai.auth_mode.api_key",
                    ),
                    LlmWizardQuestionOption::new(
                        "entra_id",
                        "wizard.llm.azure_openai.auth_mode.entra_id",
                    ),
                ],
            )
            .with_default("api_key"),
            LlmWizardQuestion::secret_ref(
                "api_key_secret",
                "wizard.llm.azure_openai.api_key_secret",
                false,
            )
            .visible_when_equals("auth_mode", "api_key"),
            LlmWizardQuestion::secret_ref(
                "entra_token_secret",
                "wizard.llm.azure_openai.entra_token_secret",
                false,
            )
            .visible_when_equals("auth_mode", "entra_id"),
            LlmWizardQuestion::text("entra_scope", "wizard.llm.azure_openai.entra_scope", false)
                .with_default("https://cognitiveservices.azure.com/.default")
                .visible_when_equals("auth_mode", "entra_id"),
            LlmWizardQuestion::boolean(
                "use_responses_api",
                "wizard.llm.azure_openai.use_responses_api",
                true,
            )
            .with_default("true"),
            LlmWizardQuestion::boolean("allow_tools", "wizard.llm.shared.allow_tools", true)
                .with_default("true"),
            LlmWizardQuestion::boolean(
                "allow_structured_outputs",
                "wizard.llm.shared.allow_structured_outputs",
                true,
            )
            .with_default("true"),
            LlmWizardQuestion::boolean(
                "allow_stateful_responses",
                "wizard.llm.azure_openai.allow_stateful_responses",
                true,
            )
            .with_default("true")
            .visible_when_equals("use_responses_api", "true"),
            LlmWizardQuestion::number("timeout_ms", "wizard.llm.shared.timeout_ms", true)
                .with_default("60000"),
        ],
    )
}

/// Returns the provider-owned wizard metadata for generic OpenAI-compatible setup.
#[must_use]
pub fn openai_compatible_llm_wizard_qa() -> LlmWizardProviderQa {
    LlmWizardProviderQa::new(
        OPENAI_COMPATIBLE_LLM_PROVIDER_NAME,
        "wizard.llm.provider.openai_compatible.label",
        false,
        vec![
            "compatible".to_string(),
            "gateway".to_string(),
            "self_hosted".to_string(),
        ],
        vec![
            LlmWizardQuestion::text("base_url", "wizard.llm.shared.base_url", true),
            LlmWizardQuestion::text("model", "wizard.llm.shared.model", true),
            LlmWizardQuestion::secret_ref(
                "api_key_secret",
                "wizard.llm.openai_compatible.api_key_secret",
                false,
            ),
            LlmWizardQuestion::select(
                "compat_mode",
                "wizard.llm.openai_compatible.compat_mode",
                true,
                vec![
                    LlmWizardQuestionOption::new(
                        "responses",
                        "wizard.llm.openai_compatible.compat_mode.responses",
                    ),
                    LlmWizardQuestionOption::new(
                        "chat_completions",
                        "wizard.llm.openai_compatible.compat_mode.chat_completions",
                    ),
                ],
            )
            .with_default("responses"),
            LlmWizardQuestion::boolean(
                "supports_stateful_responses",
                "wizard.llm.openai_compatible.supports_stateful_responses",
                true,
            )
            .with_default("false"),
            LlmWizardQuestion::boolean(
                "supports_structured_outputs",
                "wizard.llm.openai_compatible.supports_structured_outputs",
                true,
            )
            .with_default("false"),
        ],
    )
}

/// Returns the provider-owned wizard metadata for Anthropic setup.
#[must_use]
pub fn anthropic_llm_wizard_qa() -> LlmWizardProviderQa {
    LlmWizardProviderQa::new(
        ANTHROPIC_LLM_PROVIDER_NAME,
        "wizard.llm.provider.anthropic.label",
        false,
        vec!["native".to_string(), "saas".to_string()],
        vec![
            LlmWizardQuestion::secret_ref(
                "api_key_secret",
                "wizard.llm.anthropic.api_key_secret",
                true,
            ),
            LlmWizardQuestion::text("model", "wizard.llm.shared.model", true),
            LlmWizardQuestion::text("base_url", "wizard.llm.shared.base_url", false),
            LlmWizardQuestion::number("max_tokens", "wizard.llm.anthropic.max_tokens", true)
                .with_default("4096"),
            LlmWizardQuestion::number("timeout_ms", "wizard.llm.shared.timeout_ms", true)
                .with_default("60000"),
            LlmWizardQuestion::boolean("allow_tools", "wizard.llm.shared.allow_tools", true)
                .with_default("true"),
            LlmWizardQuestion::boolean(
                "allow_structured_outputs",
                "wizard.llm.shared.allow_structured_outputs",
                true,
            )
            .with_default("true"),
            LlmWizardQuestion::boolean(
                "allow_thinking",
                "wizard.llm.anthropic.allow_thinking",
                true,
            )
            .with_default("false"),
            LlmWizardQuestion::number(
                "thinking_budget_tokens",
                "wizard.llm.anthropic.thinking_budget_tokens",
                false,
            )
            .visible_when_equals("allow_thinking", "true"),
        ],
    )
}

/// Returns the provider-owned wizard metadata for Gemini setup.
#[must_use]
pub fn gemini_llm_wizard_qa() -> LlmWizardProviderQa {
    LlmWizardProviderQa::new(
        GEMINI_LLM_PROVIDER_NAME,
        "wizard.llm.provider.gemini.label",
        false,
        vec!["native".to_string(), "saas".to_string()],
        vec![
            LlmWizardQuestion::secret_ref(
                "api_key_secret",
                "wizard.llm.gemini.api_key_secret",
                true,
            ),
            LlmWizardQuestion::text("model", "wizard.llm.shared.model", true),
            LlmWizardQuestion::text("base_url", "wizard.llm.shared.base_url", false),
            LlmWizardQuestion::number("timeout_ms", "wizard.llm.shared.timeout_ms", true)
                .with_default("60000"),
            LlmWizardQuestion::boolean("allow_tools", "wizard.llm.shared.allow_tools", true)
                .with_default("true"),
            LlmWizardQuestion::boolean(
                "allow_structured_outputs",
                "wizard.llm.shared.allow_structured_outputs",
                true,
            )
            .with_default("true"),
            LlmWizardQuestion::text("safety_profile", "wizard.llm.gemini.safety_profile", false),
        ],
    )
}

/// Returns the provider-owned wizard metadata for Bedrock setup.
#[must_use]
pub fn bedrock_llm_wizard_qa() -> LlmWizardProviderQa {
    LlmWizardProviderQa::new(
        BEDROCK_LLM_PROVIDER_NAME,
        "wizard.llm.provider.bedrock.label",
        false,
        vec!["aws".to_string(), "enterprise".to_string()],
        vec![
            LlmWizardQuestion::text("region", "wizard.llm.bedrock.region", true),
            LlmWizardQuestion::text("model_id", "wizard.llm.bedrock.model_id", true),
            LlmWizardQuestion::select(
                "auth_mode",
                "wizard.llm.bedrock.auth_mode",
                true,
                vec![
                    LlmWizardQuestionOption::new(
                        "default_chain",
                        "wizard.llm.bedrock.auth_mode.default_chain",
                    ),
                    LlmWizardQuestionOption::new("profile", "wizard.llm.bedrock.auth_mode.profile"),
                    LlmWizardQuestionOption::new(
                        "static_keys",
                        "wizard.llm.bedrock.auth_mode.static_keys",
                    ),
                ],
            )
            .with_default("default_chain"),
            LlmWizardQuestion::text("profile_name", "wizard.llm.bedrock.profile_name", false)
                .visible_when_equals("auth_mode", "profile"),
            LlmWizardQuestion::secret_ref(
                "access_key_id_secret",
                "wizard.llm.bedrock.access_key_id_secret",
                false,
            )
            .visible_when_equals("auth_mode", "static_keys"),
            LlmWizardQuestion::secret_ref(
                "secret_access_key_secret",
                "wizard.llm.bedrock.secret_access_key_secret",
                false,
            )
            .visible_when_equals("auth_mode", "static_keys"),
            LlmWizardQuestion::secret_ref(
                "session_token_secret",
                "wizard.llm.bedrock.session_token_secret",
                false,
            )
            .visible_when_equals("auth_mode", "static_keys"),
            LlmWizardQuestion::number("timeout_ms", "wizard.llm.shared.timeout_ms", true)
                .with_default("60000"),
            LlmWizardQuestion::boolean("allow_tools", "wizard.llm.shared.allow_tools", true)
                .with_default("true"),
            LlmWizardQuestion::boolean(
                "allow_streaming",
                "wizard.llm.bedrock.allow_streaming",
                true,
            )
            .with_default("true"),
        ],
    )
}

/// Returns the provider-owned wizard metadata for NVIDIA NIM setup.
#[must_use]
pub fn nvidia_nim_llm_wizard_qa() -> LlmWizardProviderQa {
    LlmWizardProviderQa::new(
        NVIDIA_NIM_LLM_PROVIDER_NAME,
        "wizard.llm.provider.nvidia_nim.label",
        false,
        vec![
            "nvidia".to_string(),
            "self_hosted".to_string(),
            "compatible".to_string(),
        ],
        vec![
            LlmWizardQuestion::text("base_url", "wizard.llm.nim.base_url", true),
            LlmWizardQuestion::select(
                "api_mode",
                "wizard.llm.nim.api_mode",
                true,
                vec![
                    LlmWizardQuestionOption::new(
                        "openai_compatible",
                        "wizard.llm.nim.api_mode.openai_compatible",
                    ),
                    LlmWizardQuestionOption::new("nim_aware", "wizard.llm.nim.api_mode.nim_aware"),
                ],
            )
            .with_default("nim_aware"),
            LlmWizardQuestion::text("model", "wizard.llm.shared.model", true),
            LlmWizardQuestion::secret_ref("api_key_secret", "wizard.llm.nim.api_key_secret", false),
            LlmWizardQuestion::boolean(
                "discover_models_on_start",
                "wizard.llm.nim.discover_models_on_start",
                true,
            )
            .with_default("true")
            .visible_when_equals("api_mode", "nim_aware"),
            LlmWizardQuestion::boolean(
                "healthcheck_on_start",
                "wizard.llm.nim.healthcheck_on_start",
                true,
            )
            .with_default("true")
            .visible_when_equals("api_mode", "nim_aware"),
            LlmWizardQuestion::boolean("allow_tools", "wizard.llm.shared.allow_tools", true)
                .with_default("true"),
            LlmWizardQuestion::boolean(
                "allow_structured_outputs",
                "wizard.llm.shared.allow_structured_outputs",
                true,
            )
            .with_default("true"),
            LlmWizardQuestion::boolean("allow_streaming", "wizard.llm.nim.allow_streaming", true)
                .with_default("true"),
            LlmWizardQuestion::number("timeout_ms", "wizard.llm.shared.timeout_ms", true)
                .with_default("60000"),
        ],
    )
}

/// Returns provider-owned wizard metadata for every implemented LLM backend.
#[must_use]
pub fn implemented_llm_wizard_qas() -> Vec<LlmWizardProviderQa> {
    vec![
        openai_llm_wizard_qa(),
        azure_openai_llm_wizard_qa(),
        openai_compatible_llm_wizard_qa(),
        anthropic_llm_wizard_qa(),
        gemini_llm_wizard_qa(),
        bedrock_llm_wizard_qa(),
        nvidia_nim_llm_wizard_qa(),
    ]
}

/// Errors produced while building LLM fixtures.
#[derive(Debug, Error)]
pub enum LlmFixtureError {
    /// Capability identifier parsing failed.
    #[error(transparent)]
    CapabilityId(#[from] CapabilityIdError),
    /// Pack capability extension construction failed.
    #[error(transparent)]
    CapabilitiesExtension(#[from] CapabilitiesExtensionError),
    /// Pack CBOR encoding failed.
    #[error(transparent)]
    Cbor(#[from] CborError),
}

/// Returns the canonical LLM capability URI.
#[must_use]
pub const fn llm_capability_uri() -> &'static str {
    LLM_CAPABILITY_URI
}

/// Returns the canonical LLM capability identifier.
pub fn llm_capability_id() -> Result<greentic_cap_types::CapabilityId, CapabilityIdError> {
    greentic_cap_types::CapabilityId::new(LLM_CAPABILITY_URI)
}

/// Returns the canonical shared LLM pack capability identifier.
#[must_use]
pub const fn llm_pack_capability_id() -> &'static str {
    LLM_PACK_CAPABILITY_ID
}

/// Returns a provider-specific pack capability identifier for future LLM providers.
#[must_use]
pub fn llm_provider_pack_capability_id(provider_name: impl AsRef<str>) -> String {
    format!("{}.{}", llm_pack_capability_id(), provider_name.as_ref())
}

/// Returns the canonical provider id for an LLM backend.
#[must_use]
pub fn llm_provider_id(provider_name: impl AsRef<str>) -> String {
    crate::provider_type(ProviderCategory::Llm, provider_name)
}

/// Returns the canonical provider id for the native OpenAI backend.
#[must_use]
pub fn openai_llm_provider_id() -> String {
    llm_provider_id(OPENAI_LLM_PROVIDER_NAME)
}

/// Returns the canonical provider id for the Azure OpenAI backend.
#[must_use]
pub fn azure_openai_llm_provider_id() -> String {
    llm_provider_id(AZURE_OPENAI_LLM_PROVIDER_NAME)
}

/// Returns the canonical provider id for the Anthropic backend.
#[must_use]
pub fn anthropic_llm_provider_id() -> String {
    llm_provider_id(ANTHROPIC_LLM_PROVIDER_NAME)
}

/// Returns the canonical provider id for the Bedrock backend.
#[must_use]
pub fn bedrock_llm_provider_id() -> String {
    llm_provider_id(BEDROCK_LLM_PROVIDER_NAME)
}

/// Returns the canonical provider id for the generic OpenAI-compatible backend.
#[must_use]
pub fn openai_compatible_llm_provider_id() -> String {
    llm_provider_id(OPENAI_COMPATIBLE_LLM_PROVIDER_NAME)
}

/// Returns the canonical provider id for the Gemini backend.
#[must_use]
pub fn gemini_llm_provider_id() -> String {
    llm_provider_id(GEMINI_LLM_PROVIDER_NAME)
}

/// Returns the canonical provider id for the NVIDIA NIM backend.
#[must_use]
pub fn nvidia_nim_llm_provider_id() -> String {
    llm_provider_id(NVIDIA_NIM_LLM_PROVIDER_NAME)
}

/// Returns the shared LLM operation names referenced by the helper module.
#[must_use]
pub const fn llm_operations() -> [&'static str; 6] {
    [
        "llm.generate",
        "llm.chat",
        "llm.stream",
        "llm.tools",
        "llm.structured",
        "llm.multimodal",
    ]
}

/// Builds a typed LLM feature profile.
#[allow(clippy::too_many_arguments)]
#[must_use]
pub const fn llm_feature_profile(
    chat: bool,
    structured_outputs: bool,
    tool_calling: bool,
    streaming: bool,
    multimodal_input: bool,
    stateful_conversation: bool,
    local_self_hosted: bool,
    enterprise_auth: bool,
) -> LlmFeatureProfile {
    LlmProviderFeatures::new(
        chat,
        structured_outputs,
        tool_calling,
        streaming,
        multimodal_input,
        stateful_conversation,
        local_self_hosted,
        enterprise_auth,
    )
}

/// Returns the default feature profile for the native OpenAI backend.
#[must_use]
pub const fn openai_llm_feature_profile() -> LlmFeatureProfile {
    llm_feature_profile(true, true, true, true, false, true, false, false)
}

/// Returns the default feature profile for the Azure OpenAI backend.
#[must_use]
pub const fn azure_openai_llm_feature_profile() -> LlmFeatureProfile {
    llm_feature_profile(true, true, true, true, false, true, false, true)
}

/// Returns the default feature profile for the Anthropic backend.
#[must_use]
pub const fn anthropic_llm_feature_profile() -> LlmFeatureProfile {
    llm_feature_profile(true, true, true, true, false, false, false, true)
}

/// Returns the default feature profile for the Bedrock backend.
#[must_use]
pub const fn bedrock_llm_feature_profile() -> LlmFeatureProfile {
    llm_feature_profile(true, false, true, true, false, false, false, true)
}

/// Returns the default feature profile for the generic OpenAI-compatible backend.
#[must_use]
pub const fn openai_compatible_llm_feature_profile() -> LlmFeatureProfile {
    llm_feature_profile(true, false, true, true, false, false, true, false)
}

/// Returns the default feature profile for the Gemini backend.
#[must_use]
pub const fn gemini_llm_feature_profile() -> LlmFeatureProfile {
    llm_feature_profile(true, true, true, false, false, false, false, false)
}

/// Returns the default feature profile for the NVIDIA NIM backend.
#[must_use]
pub const fn nvidia_nim_llm_feature_profile() -> LlmFeatureProfile {
    llm_feature_profile(true, false, true, true, false, false, true, false)
}

/// Builds a capability profile record from an LLM feature profile.
#[must_use]
pub fn llm_capability_profile(
    provider_name: impl AsRef<str>,
    features: &LlmFeatureProfile,
) -> CapabilityProfile {
    features.as_capability_profile(format!("llm.{}.features", provider_name.as_ref()))
}

/// Creates a provider manifest for an LLM backend.
#[must_use]
pub fn llm_provider_manifest(
    provider_name: impl AsRef<str>,
    features: &LlmFeatureProfile,
) -> ProviderManifest {
    let provider_name = provider_name.as_ref();
    provider_manifest(
        ProviderCategory::Llm,
        provider_name,
        vec![llm_pack_capability_id().to_string()],
        features.operations(),
        Some(format!("schemas/llm/{provider_name}.json")),
        None,
    )
}

/// Creates a provider declaration for an LLM backend.
#[must_use]
pub fn llm_provider_declaration(
    provider_name: impl AsRef<str>,
    features: &LlmFeatureProfile,
) -> ProviderDecl {
    let provider_name = provider_name.as_ref();
    provider_decl(ProviderDeclSpec {
        category: ProviderCategory::Llm,
        provider_name: provider_name.to_string(),
        capabilities: vec![llm_pack_capability_id().to_string()],
        ops: features.operations(),
        config_schema_ref: format!("schemas/llm/{provider_name}.json"),
        state_schema_ref: None,
        component_ref: format!("component:llm.{provider_name}"),
        export: "greentic_provider".to_string(),
        world: "greentic:provider/runtime".to_string(),
        docs_ref: Some(format!("docs/providers/llm/{provider_name}.md")),
    })
}

/// Creates a provider manifest for the native OpenAI backend.
#[must_use]
pub fn openai_llm_provider_manifest() -> ProviderManifest {
    llm_provider_manifest(OPENAI_LLM_PROVIDER_NAME, &openai_llm_feature_profile())
}

/// Creates a provider manifest for the Azure OpenAI backend.
#[must_use]
pub fn azure_openai_llm_provider_manifest() -> ProviderManifest {
    llm_provider_manifest(
        AZURE_OPENAI_LLM_PROVIDER_NAME,
        &azure_openai_llm_feature_profile(),
    )
}

/// Creates a provider manifest for the Anthropic backend.
#[must_use]
pub fn anthropic_llm_provider_manifest() -> ProviderManifest {
    llm_provider_manifest(
        ANTHROPIC_LLM_PROVIDER_NAME,
        &anthropic_llm_feature_profile(),
    )
}

/// Creates a provider manifest for the Bedrock backend.
#[must_use]
pub fn bedrock_llm_provider_manifest() -> ProviderManifest {
    llm_provider_manifest(BEDROCK_LLM_PROVIDER_NAME, &bedrock_llm_feature_profile())
}

/// Creates a provider declaration for the native OpenAI backend.
#[must_use]
pub fn openai_llm_provider_declaration() -> ProviderDecl {
    llm_provider_declaration(OPENAI_LLM_PROVIDER_NAME, &openai_llm_feature_profile())
}

/// Creates a provider declaration for the Azure OpenAI backend.
#[must_use]
pub fn azure_openai_llm_provider_declaration() -> ProviderDecl {
    llm_provider_declaration(
        AZURE_OPENAI_LLM_PROVIDER_NAME,
        &azure_openai_llm_feature_profile(),
    )
}

/// Creates a provider declaration for the Anthropic backend.
#[must_use]
pub fn anthropic_llm_provider_declaration() -> ProviderDecl {
    llm_provider_declaration(
        ANTHROPIC_LLM_PROVIDER_NAME,
        &anthropic_llm_feature_profile(),
    )
}

/// Creates a provider declaration for the Bedrock backend.
#[must_use]
pub fn bedrock_llm_provider_declaration() -> ProviderDecl {
    llm_provider_declaration(BEDROCK_LLM_PROVIDER_NAME, &bedrock_llm_feature_profile())
}

/// Creates a provider manifest for the generic OpenAI-compatible backend.
#[must_use]
pub fn openai_compatible_llm_provider_manifest() -> ProviderManifest {
    llm_provider_manifest(
        OPENAI_COMPATIBLE_LLM_PROVIDER_NAME,
        &openai_compatible_llm_feature_profile(),
    )
}

/// Creates a provider declaration for the generic OpenAI-compatible backend.
#[must_use]
pub fn openai_compatible_llm_provider_declaration() -> ProviderDecl {
    llm_provider_declaration(
        OPENAI_COMPATIBLE_LLM_PROVIDER_NAME,
        &openai_compatible_llm_feature_profile(),
    )
}

/// Creates a provider manifest for the Gemini backend.
#[must_use]
pub fn gemini_llm_provider_manifest() -> ProviderManifest {
    llm_provider_manifest(GEMINI_LLM_PROVIDER_NAME, &gemini_llm_feature_profile())
}

/// Creates a provider declaration for the Gemini backend.
#[must_use]
pub fn gemini_llm_provider_declaration() -> ProviderDecl {
    llm_provider_declaration(GEMINI_LLM_PROVIDER_NAME, &gemini_llm_feature_profile())
}

/// Creates a provider manifest for the NVIDIA NIM backend.
#[must_use]
pub fn nvidia_nim_llm_provider_manifest() -> ProviderManifest {
    llm_provider_manifest(
        NVIDIA_NIM_LLM_PROVIDER_NAME,
        &nvidia_nim_llm_feature_profile(),
    )
}

/// Creates a provider declaration for the NVIDIA NIM backend.
#[must_use]
pub fn nvidia_nim_llm_provider_declaration() -> ProviderDecl {
    llm_provider_declaration(
        NVIDIA_NIM_LLM_PROVIDER_NAME,
        &nvidia_nim_llm_feature_profile(),
    )
}

/// Returns the canonical provider extension payload for one or more LLM backends.
#[must_use]
pub fn llm_provider_extension_inline(
    providers: Vec<(String, LlmFeatureProfile)>,
) -> ProviderExtensionInline {
    let providers = providers
        .into_iter()
        .map(|(provider_name, features)| llm_provider_declaration(provider_name, &features))
        .collect();
    provider_extension_inline(providers)
}

/// Returns the pack manifest for the Anthropic backend.
pub fn anthropic_llm_pack_manifest(pack_id: PackId) -> Result<PackManifest, LlmFixtureError> {
    llm_pack_manifest(
        pack_id,
        ANTHROPIC_LLM_PROVIDER_NAME,
        &anthropic_llm_feature_profile(),
    )
}

/// Returns the pack manifest for the Bedrock backend.
pub fn bedrock_llm_pack_manifest(pack_id: PackId) -> Result<PackManifest, LlmFixtureError> {
    llm_pack_manifest(
        pack_id,
        BEDROCK_LLM_PROVIDER_NAME,
        &bedrock_llm_feature_profile(),
    )
}

/// Returns the shared LLM capability declaration for a provider backend.
pub fn llm_capability_declaration(
    provider_name: impl AsRef<str>,
    features: &LlmFeatureProfile,
) -> Result<CapabilityDeclaration, LlmFixtureError> {
    let provider_name = provider_name.as_ref();
    let mut offer = greentic_cap_types::CapabilityOffer::new(
        format!("offer.llm.{provider_name}"),
        llm_capability_id()?,
    );
    offer.provider = Some(capability_provider_ref(
        format!("component:llm.{provider_name}"),
        if features.chat {
            "llm.chat"
        } else {
            "llm.generate"
        },
    ));
    Ok(capability_declaration(
        vec![offer],
        Vec::new(),
        Vec::new(),
        vec![llm_capability_profile(provider_name, features)],
    ))
}

/// Returns the pack capability extension payload for an LLM backend.
#[must_use]
pub fn llm_pack_capabilities(
    provider_name: impl AsRef<str>,
) -> greentic_types::CapabilitiesExtensionV1 {
    let provider_name = provider_name.as_ref();
    greentic_types::CapabilitiesExtensionV1::new(vec![greentic_types::CapabilityOfferV1 {
        offer_id: format!("offer.llm.{provider_name}"),
        cap_id: llm_pack_capability_id().to_string(),
        version: "v1".to_string(),
        provider: greentic_types::CapabilityProviderRefV1 {
            component_ref: format!("component:llm.{provider_name}"),
            op: "llm.generate".to_string(),
        },
        scope: None,
        priority: 0,
        requires_setup: false,
        setup: None,
        applies_to: None,
    }])
}

/// Builds a pack manifest for an LLM backend.
pub fn llm_pack_manifest(
    pack_id: PackId,
    provider_name: impl AsRef<str>,
    features: &LlmFeatureProfile,
) -> Result<PackManifest, LlmFixtureError> {
    let provider_name = provider_name.as_ref();
    let provider_decl = llm_provider_declaration(provider_name, features);
    let mut manifest = PackManifest {
        schema_version: "pack-v1".to_string(),
        pack_id,
        name: Some(format!("llm {provider_name}")),
        version: Version::new(0, 5, 0),
        kind: PackKind::Provider,
        publisher: "greentic".to_string(),
        components: Vec::new(),
        flows: Vec::new(),
        dependencies: Vec::new(),
        capabilities: vec![greentic_types::ComponentCapability {
            name: llm_pack_capability_id().to_string(),
            description: Some("llm capability".to_string()),
        }],
        secret_requirements: Vec::new(),
        signatures: PackSignatures::default(),
        bootstrap: None,
        extensions: None,
    };

    manifest
        .ensure_provider_extension_inline()
        .providers
        .push(provider_decl);
    manifest
        .set_capabilities_extension_v1(llm_pack_capabilities(provider_name))
        .map_err(LlmFixtureError::from)?;
    Ok(manifest)
}

/// Builds a pack manifest for the native OpenAI backend.
pub fn openai_llm_pack_manifest(pack_id: PackId) -> Result<PackManifest, LlmFixtureError> {
    llm_pack_manifest(
        pack_id,
        OPENAI_LLM_PROVIDER_NAME,
        &openai_llm_feature_profile(),
    )
}

/// Builds a pack manifest for the Azure OpenAI backend.
pub fn azure_openai_llm_pack_manifest(pack_id: PackId) -> Result<PackManifest, LlmFixtureError> {
    llm_pack_manifest(
        pack_id,
        AZURE_OPENAI_LLM_PROVIDER_NAME,
        &azure_openai_llm_feature_profile(),
    )
}

/// Builds a pack manifest for the generic OpenAI-compatible backend.
pub fn openai_compatible_llm_pack_manifest(
    pack_id: PackId,
) -> Result<PackManifest, LlmFixtureError> {
    llm_pack_manifest(
        pack_id,
        OPENAI_COMPATIBLE_LLM_PROVIDER_NAME,
        &openai_compatible_llm_feature_profile(),
    )
}

/// Builds a pack manifest for the Gemini backend.
pub fn gemini_llm_pack_manifest(pack_id: PackId) -> Result<PackManifest, LlmFixtureError> {
    llm_pack_manifest(
        pack_id,
        GEMINI_LLM_PROVIDER_NAME,
        &gemini_llm_feature_profile(),
    )
}

/// Builds a pack manifest for the NVIDIA NIM backend.
pub fn nvidia_nim_llm_pack_manifest(pack_id: PackId) -> Result<PackManifest, LlmFixtureError> {
    llm_pack_manifest(
        pack_id,
        NVIDIA_NIM_LLM_PROVIDER_NAME,
        &nvidia_nim_llm_feature_profile(),
    )
}

/// Encodes an LLM pack manifest to CBOR bytes.
pub fn llm_pack_manifest_cbor(
    pack_id: PackId,
    provider_name: impl AsRef<str>,
    features: &LlmFeatureProfile,
) -> Result<Vec<u8>, LlmFixtureError> {
    let manifest = llm_pack_manifest(pack_id, provider_name, features)?;
    Ok(encode_pack_manifest(&manifest)?)
}
