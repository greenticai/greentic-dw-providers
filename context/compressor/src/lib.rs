#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Deterministic context compression provider.

use greentic_dw_context::{
    ContextFragment, ContextPackage, ContextProvider, ContextRequest, ContextSourceKind,
};
use greentic_types::{GResult, TenantCtx};
use serde_json::json;

/// Compression provider configuration.
#[derive(Clone, Debug, PartialEq)]
pub struct CompressorConfig {
    /// Maximum number of fragments to retain before compression.
    pub max_fragments: usize,
    /// Whether overflow should be summarized instead of dropped entirely.
    pub summarizer_configured: bool,
}

/// Deterministic context compressor over a configured input package.
pub struct CompressorContextProvider {
    input: ContextPackage,
    config: CompressorConfig,
}

impl CompressorContextProvider {
    /// Creates a compressor provider over a known input package.
    #[must_use]
    pub fn new(input: ContextPackage, config: CompressorConfig) -> Self {
        Self { input, config }
    }
}

impl ContextProvider for CompressorContextProvider {
    fn assemble(&self, _tenant: &TenantCtx, _request: ContextRequest) -> GResult<ContextPackage> {
        let mut pinned: Vec<_> = self
            .input
            .fragments
            .iter()
            .filter(|fragment| fragment.pinned)
            .cloned()
            .collect();
        let mut ranked: Vec<_> = self
            .input
            .fragments
            .iter()
            .filter(|fragment| !fragment.pinned)
            .cloned()
            .collect();
        ranked.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(left.id.cmp(&right.id))
        });

        let pinned_count = pinned.len();
        let retain_extra = self.config.max_fragments.saturating_sub(pinned_count);
        let retained_ranked: Vec<_> = ranked.iter().take(retain_extra).cloned().collect();
        let overflow: Vec<_> = ranked.iter().skip(retain_extra).cloned().collect();

        let mut fragments = Vec::new();
        fragments.append(&mut pinned);
        fragments.extend(retained_ranked);

        if self.config.summarizer_configured && !overflow.is_empty() {
            fragments.push(
                ContextFragment::new(
                    "overflow-summary",
                    json!({
                        "summary_of": overflow.iter().map(|fragment| fragment.id.clone()).collect::<Vec<_>>(),
                        "count": overflow.len()
                    }),
                )?
                .with_source_kind(ContextSourceKind::Summary)
                .with_provenance_ref("compression/overflow-summary")
                .with_score(0.0)
                .with_pinned(true),
            );
        }

        Ok(ContextPackage::empty()
            .with_metadata(json!({
                "provider": "compressor",
                "kept_count": fragments.len(),
                "overflow_count": overflow.len(),
                "truncated_without_summary": !self.config.summarizer_configured && !overflow.is_empty(),
            }))
            .with_fragments(fragments))
    }
}

trait PackageFragmentsExt {
    fn with_fragments(self, fragments: Vec<ContextFragment>) -> Self;
}

impl PackageFragmentsExt for ContextPackage {
    fn with_fragments(mut self, fragments: Vec<ContextFragment>) -> Self {
        self.fragments.extend(fragments);
        self
    }
}
