use greentic_dw_providers_common::{
    LlmWizardProviderQa, LlmWizardQuestion, azure_openai_llm_wizard_qa,
};

/// Shared wizard question descriptor for Azure OpenAI setup.
pub type AzureOpenAiWizardQuestion = LlmWizardQuestion;

/// Returns the provider-owned wizard metadata for Azure OpenAI setup.
#[must_use]
pub fn azure_openai_wizard_qa() -> LlmWizardProviderQa {
    azure_openai_llm_wizard_qa()
}

/// Returns the provider-owned wizard questions for Azure OpenAI setup.
#[must_use]
pub fn azure_openai_wizard_questions() -> Vec<AzureOpenAiWizardQuestion> {
    azure_openai_wizard_qa().questions
}
