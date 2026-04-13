use greentic_dw_providers_common::{LlmWizardProviderQa, LlmWizardQuestion, bedrock_llm_wizard_qa};

/// Shared wizard question descriptor for Bedrock setup.
pub type BedrockWizardQuestion = LlmWizardQuestion;

/// Returns the provider-owned wizard metadata for Bedrock setup.
#[must_use]
pub fn bedrock_wizard_qa() -> LlmWizardProviderQa {
    bedrock_llm_wizard_qa()
}

/// Returns the provider-owned wizard questions for Bedrock setup.
#[must_use]
pub fn bedrock_wizard_questions() -> Vec<BedrockWizardQuestion> {
    bedrock_wizard_qa().questions
}
