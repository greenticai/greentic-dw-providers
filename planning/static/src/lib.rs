#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Deterministic static planning provider.

mod config;
mod provider;

pub use config::StaticPlanningConfig;
pub use provider::StaticPlanningProvider;
