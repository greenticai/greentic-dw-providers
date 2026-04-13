use greentic_dw_providers_common::{
    LlmWizardProviderQa, LlmWizardQuestion, anthropic_llm_wizard_qa,
};

/// Shared wizard question descriptor for Anthropic setup.
pub type AnthropicWizardQuestion = LlmWizardQuestion;

/// Returns the provider-owned wizard metadata for Anthropic setup.
#[must_use]
pub fn anthropic_wizard_qa() -> LlmWizardProviderQa {
    anthropic_llm_wizard_qa()
}

/// Returns the provider-owned wizard questions for Anthropic setup.
#[must_use]
pub fn anthropic_wizard_questions() -> Vec<AnthropicWizardQuestion> {
    anthropic_wizard_qa().questions
}
