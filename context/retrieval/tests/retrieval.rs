use greentic_dw_context::ContextProvider;
use greentic_dw_context_retrieval::{RetrievalContextProvider, RetrievalRef};
use greentic_dw_planning::{PlanDocument, PlanStep};
use greentic_dw_workspace::{
    WorkspaceArtifactId, WorkspaceArtifactVersion, WorkspaceProvider, WorkspaceScope,
};
use greentic_dw_workspace_in_memory::InMemoryWorkspaceProvider;
use greentic_types::{EnvId, TenantCtx, TenantId};
use serde_json::json;

fn tenant() -> TenantCtx {
    TenantCtx::new(
        EnvId::try_from("dev").expect("env id"),
        TenantId::try_from("tenant-context").expect("tenant id"),
    )
}

#[test]
fn retrieval_preserves_provenance_refs() {
    let workspace = InMemoryWorkspaceProvider::new();
    let tenant = tenant();
    let scope = WorkspaceScope::new("analysis").expect("scope");
    let artifact = WorkspaceArtifactId::new("artifact-a").expect("artifact");
    workspace
        .write_version(
            &tenant,
            &scope,
            &artifact,
            WorkspaceArtifactVersion::new("v1", json!({"title": "workspace"})).expect("v1"),
        )
        .expect("write");

    let plan = PlanDocument::new("plan-1").expect("plan").with_step(
        PlanStep::new("step-1", "research")
            .expect("step")
            .with_payload(json!({"title": "plan"})),
    );

    let provider = RetrievalContextProvider::new(
        workspace,
        vec![
            RetrievalRef::Workspace {
                scope,
                artifact_id: artifact,
                score: 0.8,
            },
            RetrievalRef::PlanStep {
                plan,
                step_id: "step-1".to_string(),
                score: 0.7,
            },
            RetrievalRef::MemoryRef {
                reference: "memo-1".to_string(),
                content: json!({"title": "memory"}),
                score: 0.6,
            },
        ],
    );

    let package = provider
        .assemble(
            &tenant,
            greentic_dw_context::ContextRequest::new("req-1").expect("request"),
        )
        .expect("package");

    assert_eq!(package.fragments.len(), 3);
    assert!(
        package
            .fragments
            .iter()
            .all(|fragment| !fragment.provenance_ref.is_empty())
    );
}
