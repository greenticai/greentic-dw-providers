#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Filesystem workspace provider.

use greentic_dw_workspace::{
    WorkspaceArtifactId, WorkspaceArtifactVersion, WorkspaceProvider, WorkspaceScope,
    normalize_version,
};
use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Filesystem workspace configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FsWorkspaceConfig {
    /// Root directory for all stored workspace artifacts.
    pub root_dir: PathBuf,
}

impl FsWorkspaceConfig {
    /// Creates a filesystem config rooted at the supplied directory.
    #[must_use]
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: root_dir.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct FsWorkspaceMetadata {
    scope: String,
    artifact_id: String,
    version_id: String,
    provenance: greentic_dw_workspace::WorkspaceProvenance,
    metadata: greentic_dw_workspace::WorkspaceVersionMetadata,
}

/// Filesystem workspace provider.
pub struct FsWorkspaceProvider {
    config: FsWorkspaceConfig,
}

impl FsWorkspaceProvider {
    /// Creates a filesystem workspace provider rooted at the configured directory.
    pub fn new(config: FsWorkspaceConfig) -> GResult<Self> {
        fs::create_dir_all(&config.root_dir)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?;
        Ok(Self { config })
    }

    fn tenant_root(&self, tenant: &TenantCtx) -> PathBuf {
        self.config
            .root_dir
            .join(tenant.env.to_string())
            .join(tenant.tenant_id.to_string())
    }

    fn scope_dir(&self, tenant: &TenantCtx, scope: &WorkspaceScope) -> PathBuf {
        self.tenant_root(tenant).join(&scope.0)
    }

    fn artifact_dir(
        &self,
        tenant: &TenantCtx,
        scope: &WorkspaceScope,
        artifact_id: &WorkspaceArtifactId,
    ) -> PathBuf {
        self.scope_dir(tenant, scope).join(&artifact_id.0)
    }

    fn version_dir(
        &self,
        tenant: &TenantCtx,
        scope: &WorkspaceScope,
        artifact_id: &WorkspaceArtifactId,
        version_id: &str,
    ) -> PathBuf {
        self.artifact_dir(tenant, scope, artifact_id)
            .join(version_id)
    }
}

impl WorkspaceProvider for FsWorkspaceProvider {
    fn list_artifacts(
        &self,
        tenant: &TenantCtx,
        scope: &WorkspaceScope,
    ) -> GResult<Vec<WorkspaceArtifactId>> {
        let scope_dir = self.scope_dir(tenant, scope);
        if !scope_dir.exists() {
            return Ok(Vec::new());
        }
        let mut artifacts = Vec::new();
        for entry in fs::read_dir(scope_dir)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?
        {
            let entry =
                entry.map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?;
            if entry
                .file_type()
                .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?
                .is_dir()
            {
                artifacts.push(WorkspaceArtifactId::new(
                    entry.file_name().to_string_lossy().into_owned(),
                )?);
            }
        }
        artifacts.sort();
        Ok(artifacts)
    }

    fn load_history(
        &self,
        tenant: &TenantCtx,
        scope: &WorkspaceScope,
        artifact_id: &WorkspaceArtifactId,
    ) -> GResult<Vec<WorkspaceArtifactVersion>> {
        let artifact_dir = self.artifact_dir(tenant, scope, artifact_id);
        if !artifact_dir.exists() {
            return Ok(Vec::new());
        }

        let mut histories = Vec::new();
        let mut version_dirs: Vec<_> = fs::read_dir(&artifact_dir)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?;
        version_dirs.sort_by_key(|entry| entry.file_name());

        for entry in version_dirs {
            if !entry
                .file_type()
                .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?
                .is_dir()
            {
                continue;
            }
            let version_dir = entry.path();
            let metadata: FsWorkspaceMetadata = serde_json::from_slice(
                &fs::read(version_dir.join("metadata.json"))
                    .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?,
            )
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?;
            let content = serde_json::from_slice(
                &fs::read(version_dir.join("content.json"))
                    .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?,
            )
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?;

            histories.push(WorkspaceArtifactVersion {
                version_id: metadata.version_id,
                content,
                provenance: metadata.provenance,
                metadata: metadata.metadata,
            });
        }

        Ok(histories)
    }

    fn write_version(
        &self,
        tenant: &TenantCtx,
        scope: &WorkspaceScope,
        artifact_id: &WorkspaceArtifactId,
        version: WorkspaceArtifactVersion,
    ) -> GResult<()> {
        let version = normalize_version(version)?;
        let history = self.load_history(tenant, scope, artifact_id)?;
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

        let version_dir = self.version_dir(tenant, scope, artifact_id, &version.version_id);
        fs::create_dir_all(&version_dir)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?;

        let metadata = FsWorkspaceMetadata {
            scope: scope.0.clone(),
            artifact_id: artifact_id.0.clone(),
            version_id: version.version_id.clone(),
            provenance: version.provenance.clone(),
            metadata: version.metadata.clone(),
        };
        fs::write(
            version_dir.join("metadata.json"),
            serde_json::to_vec_pretty(&metadata)
                .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?,
        )
        .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?;
        fs::write(
            version_dir.join("content.json"),
            serde_json::to_vec_pretty(&version.content)
                .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?,
        )
        .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{FsWorkspaceConfig, FsWorkspaceProvider};
    use greentic_dw_workspace::{
        WorkspaceArtifactId, WorkspaceArtifactVersion, WorkspaceProvider, WorkspaceScope,
    };
    use greentic_types::{EnvId, TenantCtx, TenantId};
    use serde_json::json;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-workspace").expect("tenant id"),
        )
    }

    fn temp_root() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("dw-workspace-fs-{unique}"))
    }

    #[test]
    fn fs_provider_rejects_path_traversal() {
        assert!(WorkspaceScope::new("../escape").is_err());
        assert!(WorkspaceArtifactId::new("bad/name").is_err());
    }

    #[test]
    fn list_artifacts_by_scope_works() {
        let provider =
            FsWorkspaceProvider::new(FsWorkspaceConfig::new(temp_root())).expect("provider");
        let tenant = tenant();
        let scope = WorkspaceScope::new("analysis").expect("scope");
        let artifact = WorkspaceArtifactId::new("doc").expect("artifact");

        provider
            .write_version(
                &tenant,
                &scope,
                &artifact,
                WorkspaceArtifactVersion::new("v1", json!({"title": "draft"})).expect("v1"),
            )
            .expect("write");

        let listed = provider.list_artifacts(&tenant, &scope).expect("list");
        assert_eq!(listed, vec![artifact]);
    }

    #[test]
    fn artifact_provenance_survives_reload() {
        let root = temp_root();
        let scope = WorkspaceScope::new("analysis").expect("scope");
        let artifact = WorkspaceArtifactId::new("doc").expect("artifact");
        let tenant = tenant();

        {
            let provider =
                FsWorkspaceProvider::new(FsWorkspaceConfig::new(root.clone())).expect("provider");
            provider
                .write_version(
                    &tenant,
                    &scope,
                    &artifact,
                    WorkspaceArtifactVersion::new("v1", json!({"title": "draft"}))
                        .expect("v1")
                        .with_provenance(None::<String>, Some("planner")),
                )
                .expect("v1 write");
            provider
                .write_version(
                    &tenant,
                    &scope,
                    &artifact,
                    WorkspaceArtifactVersion::new("v2", json!({"title": "final"}))
                        .expect("v2")
                        .with_provenance(Some("v1"), Some("reviewer")),
                )
                .expect("v2 write");
        }

        let reloaded =
            FsWorkspaceProvider::new(FsWorkspaceConfig::new(root)).expect("reloaded provider");
        let history = reloaded
            .load_history(&tenant, &scope, &artifact)
            .expect("history");
        assert_eq!(history.len(), 2);
        assert_eq!(history[1].provenance.derived_from.as_deref(), Some("v1"));
        assert!(!history[1].metadata.content_checksum.is_empty());
    }
}
