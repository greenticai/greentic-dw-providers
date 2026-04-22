use greentic_dw_context::{ContextFragment, ContextProvider, ContextRequest, ContextSourceKind};
use greentic_dw_context_static::{StaticContextConfig, StaticContextProvider};
use greentic_types::{EnvId, TenantCtx, TenantId};
use serde_json::json;

fn tenant() -> TenantCtx {
    TenantCtx::new(
        EnvId::try_from("dev").expect("env id"),
        TenantId::try_from("tenant-context").expect("tenant id"),
    )
}

#[test]
fn fragment_ordering_deterministic() {
    let provider = StaticContextProvider::new(StaticContextConfig {
        fragments: vec![
            ContextFragment::new("a", json!({"rank": 1}))
                .expect("a")
                .with_source_kind(ContextSourceKind::Static)
                .with_score(0.5),
            ContextFragment::new("b", json!({"rank": 2}))
                .expect("b")
                .with_source_kind(ContextSourceKind::Static)
                .with_score(0.4),
        ],
        include_runtime_metadata: false,
    });

    let package = provider
        .assemble(&tenant(), ContextRequest::new("req-1").expect("request"))
        .expect("package");

    assert_eq!(package.fragments[0].id, "a");
    assert_eq!(package.fragments[1].id, "b");
}
