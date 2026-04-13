use greentic_dw_providers_common::{
    LlmWizardProviderQa, LlmWizardQuestion, nvidia_nim_llm_wizard_qa,
};

/// Shared wizard question descriptor for NVIDIA NIM setup.
pub type NvidiaNimWizardQuestion = LlmWizardQuestion;

/// Returns the provider-owned wizard metadata for NVIDIA NIM setup.
#[must_use]
pub fn nvidia_nim_wizard_qa() -> LlmWizardProviderQa {
    nvidia_nim_llm_wizard_qa()
}

/// Returns the provider-owned wizard questions for NVIDIA NIM setup.
#[must_use]
pub fn nvidia_nim_wizard_questions() -> Vec<NvidiaNimWizardQuestion> {
    nvidia_nim_wizard_qa().questions
}
