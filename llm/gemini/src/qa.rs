use greentic_dw_providers_common::{LlmWizardProviderQa, LlmWizardQuestion, gemini_llm_wizard_qa};

/// Shared wizard question descriptor for Gemini setup.
pub type GeminiWizardQuestion = LlmWizardQuestion;

/// Returns the provider-owned wizard metadata for Gemini setup.
#[must_use]
pub fn gemini_wizard_qa() -> LlmWizardProviderQa {
    gemini_llm_wizard_qa()
}

/// Returns the provider-owned wizard questions for Gemini setup.
#[must_use]
pub fn gemini_wizard_questions() -> Vec<GeminiWizardQuestion> {
    gemini_wizard_qa().questions
}
