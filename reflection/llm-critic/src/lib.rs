#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Strict typed LLM review backend.

use greentic_dw_reflection::{ReflectionProvider, ReviewOutcome, ReviewRequest};
use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};

/// Minimal model adapter used by the LLM critic.
pub trait CriticModel: Send + Sync {
    /// Generates a JSON review outcome for the supplied prompt.
    fn generate_review_json(&self, prompt: &str) -> GResult<String>;
}

/// Configuration for the LLM critic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LlmCriticConfig {
    /// Stable reference to the underlying LLM provider.
    pub llm_provider_ref: String,
}

impl LlmCriticConfig {
    /// Creates a critic config.
    pub fn new(llm_provider_ref: impl Into<String>) -> Self {
        Self {
            llm_provider_ref: llm_provider_ref.into(),
        }
    }
}

/// Typed LLM review backend.
pub struct LlmCriticReflectionProvider<M> {
    config: LlmCriticConfig,
    model: M,
}

impl<M> LlmCriticReflectionProvider<M> {
    /// Creates a new LLM critic provider.
    pub fn new(config: LlmCriticConfig, model: M) -> GResult<Self> {
        if config.llm_provider_ref.trim().is_empty() {
            return Err(invalid("llm provider ref must not be empty"));
        }
        Ok(Self { config, model })
    }
}

impl<M: CriticModel> ReflectionProvider for LlmCriticReflectionProvider<M> {
    fn review(&self, _tenant: &TenantCtx, request: ReviewRequest) -> GResult<ReviewOutcome> {
        let prompt = build_prompt(&self.config, &request);
        let first = self.model.generate_review_json(&prompt)?;
        match parse_outcome(&first) {
            Ok(outcome) => Ok(outcome),
            Err(err) => {
                let retry = self.model.generate_review_json(&format!(
                    "{prompt}\nThe previous response was invalid: {}. Return corrected JSON only.",
                    err
                ))?;
                parse_outcome(&retry)
            }
        }
    }
}

fn build_prompt(config: &LlmCriticConfig, request: &ReviewRequest) -> String {
    format!(
        "You are the reflection backend for {}. Return only JSON for ReviewOutcome with \
         `disposition` and `findings`. Subject: {}",
        config.llm_provider_ref, request.subject
    )
}

fn parse_outcome(candidate: &str) -> GResult<ReviewOutcome> {
    let outcome: ReviewOutcome = serde_json::from_str(candidate)
        .map_err(|err| invalid(format!("critic returned invalid review json: {err}")))?;
    outcome.validate()?;
    Ok(outcome)
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}
