use greentic_dw_context::{
    ContextFragment, ContextPackage, ContextProvider, ContextRequest, ContextSourceKind,
};
use greentic_dw_context_compressor::{CompressorConfig, CompressorContextProvider};
use greentic_types::{EnvId, TenantCtx, TenantId};
use serde_json::json;

fn tenant() -> TenantCtx {
    TenantCtx::new(
        EnvId::try_from("dev").expect("env id"),
        TenantId::try_from("tenant-context").expect("tenant id"),
    )
}

#[test]
fn pinned_fragments_are_never_dropped() {
    let package = ContextPackage::empty()
        .with_fragment(
            ContextFragment::new("pinned", json!({"kind": "pinned"}))
                .expect("pinned")
                .with_source_kind(ContextSourceKind::Static)
                .with_score(0.1)
                .with_pinned(true),
        )
        .with_fragment(
            ContextFragment::new("ranked-a", json!({"kind": "a"}))
                .expect("a")
                .with_source_kind(ContextSourceKind::Static)
                .with_score(0.9),
        )
        .with_fragment(
            ContextFragment::new("ranked-b", json!({"kind": "b"}))
                .expect("b")
                .with_source_kind(ContextSourceKind::Static)
                .with_score(0.8),
        );
    let provider = CompressorContextProvider::new(
        package,
        CompressorConfig {
            max_fragments: 2,
            summarizer_configured: false,
        },
    );

    let compressed = provider
        .assemble(&tenant(), ContextRequest::new("req-1").expect("request"))
        .expect("compressed");
    assert!(
        compressed
            .fragments
            .iter()
            .any(|fragment| fragment.id == "pinned")
    );
}

#[test]
fn overflow_produces_compression_metadata() {
    let package = ContextPackage::empty()
        .with_fragment(
            ContextFragment::new("a", json!({"kind": "a"}))
                .expect("a")
                .with_source_kind(ContextSourceKind::Static)
                .with_score(0.9),
        )
        .with_fragment(
            ContextFragment::new("b", json!({"kind": "b"}))
                .expect("b")
                .with_source_kind(ContextSourceKind::Static)
                .with_score(0.8),
        )
        .with_fragment(
            ContextFragment::new("c", json!({"kind": "c"}))
                .expect("c")
                .with_source_kind(ContextSourceKind::Static)
                .with_score(0.7),
        );
    let provider = CompressorContextProvider::new(
        package,
        CompressorConfig {
            max_fragments: 2,
            summarizer_configured: true,
        },
    );

    let compressed = provider
        .assemble(&tenant(), ContextRequest::new("req-2").expect("request"))
        .expect("compressed");
    assert_eq!(compressed.metadata["overflow_count"], json!(1));
    assert!(
        compressed
            .fragments
            .iter()
            .any(|fragment| fragment.id == "overflow-summary")
    );
}
