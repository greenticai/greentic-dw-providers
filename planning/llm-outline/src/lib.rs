#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Strict structured-output planning provider.

mod config;
mod prompt;
mod provider;

pub use config::{LlmOutlineConfig, ReplanPromptVariant, SchemaStrictness};
pub use provider::{LlmOutlinePlanningProvider, OutlineModel};
