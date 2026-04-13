use greentic_dw_providers_common::{
    LlmWizardProviderQa, LlmWizardQuestion, openai_compatible_llm_wizard_qa,
};

/// Shared wizard question descriptor for generic OpenAI-compatible setup.
pub type OpenAiCompatibleWizardQuestion = LlmWizardQuestion;

/// Returns the provider-owned wizard metadata for generic compatibility setup.
#[must_use]
pub fn openai_compatible_wizard_qa() -> LlmWizardProviderQa {
    openai_compatible_llm_wizard_qa()
}

/// Returns the provider-owned wizard questions for generic compatibility setup.
#[must_use]
pub fn openai_compatible_wizard_questions() -> Vec<OpenAiCompatibleWizardQuestion> {
    openai_compatible_wizard_qa().questions
}
