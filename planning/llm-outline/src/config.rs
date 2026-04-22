/// How strictly the provider should validate the returned JSON plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaStrictness {
    /// Parse JSON and validate the normalized plan shape.
    Strict,
    /// Keep the same validation flow but allow relaxed prompting language.
    Relaxed,
}

/// Prompt variant used when asking for a replan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplanPromptVariant {
    /// Retry with direct validator feedback.
    ValidatorFeedback,
    /// Retry with a short reminder to return JSON only.
    JsonRepair,
}

/// Configuration for the outline planner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LlmOutlineConfig {
    /// Stable reference to the underlying LLM provider.
    pub llm_provider_ref: String,
    /// Maximum allowed step count.
    pub max_steps: usize,
    /// Optional allow-list of step kinds.
    pub allowed_step_kinds: Vec<String>,
    /// Validation strictness.
    pub schema_strictness: SchemaStrictness,
    /// Prompt style used when asking for a corrected plan.
    pub replan_prompt_variant: ReplanPromptVariant,
}

impl LlmOutlineConfig {
    /// Creates a new outline config.
    pub fn new(llm_provider_ref: impl Into<String>) -> Self {
        Self {
            llm_provider_ref: llm_provider_ref.into(),
            max_steps: 8,
            allowed_step_kinds: Vec::new(),
            schema_strictness: SchemaStrictness::Strict,
            replan_prompt_variant: ReplanPromptVariant::ValidatorFeedback,
        }
    }
}
