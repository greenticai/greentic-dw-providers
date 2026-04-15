#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! In-memory workspace provider.

use greentic_dw_workspace::{
    WorkspaceArtifactId, WorkspaceArtifactVersion, WorkspaceProvider, WorkspaceScope,
    normalize_version,
};
use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};
use std::collections::BTreeMap;
use std::sync::RwLock;

type ArtifactHistory = Vec<WorkspaceArtifactVersion>;
type ScopeArtifacts = BTreeMap<WorkspaceArtifactId, ArtifactHistory>;
type TenantWorkspace = BTreeMap<WorkspaceScope, ScopeArtifacts>;

/// In-memory workspace provider with immutable version history.
#[derive(Default)]
pub struct InMemoryWorkspaceProvider {
    data: RwLock<BTreeMap<String, TenantWorkspace>>,
}

impl InMemoryWorkspaceProvider {
    /// Creates an empty in-memory workspace provider.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn tenant_key(tenant: &TenantCtx) -> String {
        format!("{}::{}", tenant.env, tenant.tenant_id)
    }
}

impl WorkspaceProvider for InMemoryWorkspaceProvider {
    fn list_artifacts(
        &self,
        tenant: &TenantCtx,
        scope: &WorkspaceScope,
    ) -> GResult<Vec<WorkspaceArtifactId>> {
        let data = self
            .data
            .read()
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?;
        Ok(data
            .get(&Self::tenant_key(tenant))
            .and_then(|tenant_workspace| tenant_workspace.get(scope))
            .map(|artifacts| artifacts.keys().cloned().collect())
            .unwrap_or_default())
    }

    fn load_history(
        &self,
        tenant: &TenantCtx,
        scope: &WorkspaceScope,
        artifact_id: &WorkspaceArtifactId,
    ) -> GResult<Vec<WorkspaceArtifactVersion>> {
        let data = self
            .data
            .read()
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?;
        Ok(data
            .get(&Self::tenant_key(tenant))
            .and_then(|tenant_workspace| tenant_workspace.get(scope))
            .and_then(|artifacts| artifacts.get(artifact_id))
            .cloned()
            .unwrap_or_default())
    }

    fn write_version(
        &self,
        tenant: &TenantCtx,
        scope: &WorkspaceScope,
        artifact_id: &WorkspaceArtifactId,
        version: WorkspaceArtifactVersion,
    ) -> GResult<()> {
        let version = normalize_version(version)?;
        let mut data = self
            .data
            .write()
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?;
        let tenant_workspace = data.entry(Self::tenant_key(tenant)).or_default();
        let artifacts = tenant_workspace.entry(scope.clone()).or_default();
        let history = artifacts.entry(artifact_id.clone()).or_default();

        if history
            .iter()
            .any(|existing| existing.version_id == version.version_id)
        {
            return Err(GreenticError::new(
                ErrorCode::InvalidInput,
                format!("workspace version `{}` already exists", version.version_id),
            ));
        }

        if let Some(derived_from) = &version.provenance.derived_from
            && !history
                .iter()
                .any(|existing| &existing.version_id == derived_from)
        {
            return Err(GreenticError::new(
                ErrorCode::InvalidInput,
                format!("derived_from `{derived_from}` does not exist in history"),
            ));
        }

        history.push(version);
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::InMemoryWorkspaceProvider;
    use greentic_dw_workspace::{
        WorkspaceArtifactId, WorkspaceArtifactVersion, WorkspaceProvider, WorkspaceScope,
    };
    use greentic_types::{EnvId, TenantCtx, TenantId};
    use serde_json::json;

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-workspace").expect("tenant id"),
        )
    }

    #[test]
    fn version_history_preserved() {
        let provider = InMemoryWorkspaceProvider::new();
        let scope = WorkspaceScope::new("analysis").expect("scope");
        let artifact = WorkspaceArtifactId::new("plan-doc").expect("artifact");
        let tenant = tenant();

        provider
            .write_version(
                &tenant,
                &scope,
                &artifact,
                WorkspaceArtifactVersion::new("v1", json!({"status": "draft"})).expect("v1"),
            )
            .expect("write v1");
        provider
            .write_version(
                &tenant,
                &scope,
                &artifact,
                WorkspaceArtifactVersion::new("v2", json!({"status": "final"}))
                    .expect("v2")
                    .with_provenance(Some("v1"), Some("planner")),
            )
            .expect("write v2");

        let history = provider
            .load_history(&tenant, &scope, &artifact)
            .expect("history");
        assert_eq!(history.len(), 2);
        assert_eq!(history[1].provenance.derived_from.as_deref(), Some("v1"));
    }

    #[test]
    fn list_artifacts_by_scope_works() {
        let provider = InMemoryWorkspaceProvider::new();
        let tenant = tenant();
        let analysis = WorkspaceScope::new("analysis").expect("analysis");
        let review = WorkspaceScope::new("review").expect("review");

        provider
            .write_version(
                &tenant,
                &analysis,
                &WorkspaceArtifactId::new("artifact-a").expect("a"),
                WorkspaceArtifactVersion::new("v1", json!({"ok": true})).expect("v1"),
            )
            .expect("analysis write");
        provider
            .write_version(
                &tenant,
                &review,
                &WorkspaceArtifactId::new("artifact-b").expect("b"),
                WorkspaceArtifactVersion::new("v1", json!({"ok": true})).expect("v1"),
            )
            .expect("review write");

        let listed = provider.list_artifacts(&tenant, &analysis).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].0, "artifact-a");
    }
}
