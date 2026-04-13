use greentic_dw_providers_common::{LlmWizardProviderQa, LlmWizardQuestion, openai_llm_wizard_qa};

/// Shared wizard question descriptor for native OpenAI setup.
pub type OpenAiWizardQuestion = LlmWizardQuestion;

/// Returns the provider-owned wizard metadata for native OpenAI setup.
#[must_use]
pub fn openai_wizard_qa() -> LlmWizardProviderQa {
    openai_llm_wizard_qa()
}

/// Returns the provider-owned wizard questions for native OpenAI setup.
#[must_use]
pub fn openai_wizard_questions() -> Vec<OpenAiWizardQuestion> {
    openai_wizard_qa().questions
}
