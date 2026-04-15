#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Shared workspace artifact contract and provenance models.

use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A stable workspace scope identifier.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WorkspaceScope(pub String);

impl WorkspaceScope {
    /// Creates a validated workspace scope.
    pub fn new(value: impl Into<String>) -> GResult<Self> {
        let value = value.into();
        validate_segment("workspace scope", &value)?;
        Ok(Self(value))
    }
}

/// A stable artifact identifier inside a workspace.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct WorkspaceArtifactId(pub String);

impl WorkspaceArtifactId {
    /// Creates a validated artifact id.
    pub fn new(value: impl Into<String>) -> GResult<Self> {
        let value = value.into();
        validate_segment("workspace artifact id", &value)?;
        Ok(Self(value))
    }
}

/// Provenance for a materialized workspace artifact version.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceProvenance {
    /// Optional source artifact version id.
    pub derived_from: Option<String>,
    /// Optional actor or provider identifier.
    pub source: Option<String>,
}

/// Materialized metadata for an artifact version.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceVersionMetadata {
    /// Stable content checksum for the serialized artifact content.
    pub content_checksum: String,
    /// Free-form metadata for the version.
    pub attributes: Map<String, Value>,
}

/// A single materialized artifact version.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkspaceArtifactVersion {
    /// Stable artifact version id.
    pub version_id: String,
    /// Opaque artifact content.
    pub content: Value,
    /// Provenance metadata.
    pub provenance: WorkspaceProvenance,
    /// Version metadata including persisted checksum.
    pub metadata: WorkspaceVersionMetadata,
}

impl WorkspaceArtifactVersion {
    /// Creates a new artifact version.
    pub fn new(version_id: impl Into<String>, content: Value) -> GResult<Self> {
        let version_id = version_id.into();
        if version_id.trim().is_empty() {
            return Err(invalid("workspace version id must not be empty"));
        }

        Ok(Self {
            version_id,
            content,
            provenance: WorkspaceProvenance {
                derived_from: None,
                source: None,
            },
            metadata: WorkspaceVersionMetadata {
                content_checksum: String::new(),
                attributes: Map::new(),
            },
        })
    }

    /// Sets provenance information on the version.
    #[must_use]
    pub fn with_provenance(
        mut self,
        derived_from: Option<impl Into<String>>,
        source: Option<impl Into<String>>,
    ) -> Self {
        self.provenance = WorkspaceProvenance {
            derived_from: derived_from.map(Into::into),
            source: source.map(Into::into),
        };
        self
    }

    /// Adds free-form metadata.
    #[must_use]
    pub fn with_attribute(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.attributes.insert(key.into(), value);
        self
    }
}

/// Contract implemented by workspace backends.
pub trait WorkspaceProvider: Send + Sync {
    /// Lists artifact ids available for the given tenant and scope.
    fn list_artifacts(
        &self,
        tenant: &TenantCtx,
        scope: &WorkspaceScope,
    ) -> GResult<Vec<WorkspaceArtifactId>>;

    /// Loads the full immutable version history for an artifact.
    fn load_history(
        &self,
        tenant: &TenantCtx,
        scope: &WorkspaceScope,
        artifact_id: &WorkspaceArtifactId,
    ) -> GResult<Vec<WorkspaceArtifactVersion>>;

    /// Stores a new artifact version.
    fn write_version(
        &self,
        tenant: &TenantCtx,
        scope: &WorkspaceScope,
        artifact_id: &WorkspaceArtifactId,
        version: WorkspaceArtifactVersion,
    ) -> GResult<()>;
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}

fn validate_segment(label: &str, value: &str) -> GResult<()> {
    if value.trim().is_empty() {
        return Err(invalid(format!("{label} must not be empty")));
    }
    if value.starts_with('.') || value.contains("..") || value.contains('/') || value.contains('\\')
    {
        return Err(invalid(format!(
            "{label} must not contain path traversal or separators"
        )));
    }
    Ok(())
}

/// Computes a stable checksum for serialized workspace content.
pub fn content_checksum(content: &Value) -> GResult<String> {
    let encoded = serde_json::to_vec(content)
        .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?;
    Ok(blake3::hash(&encoded).to_hex().to_string())
}

/// Validates and normalizes a version before persistence.
pub fn normalize_version(version: WorkspaceArtifactVersion) -> GResult<WorkspaceArtifactVersion> {
    if version.version_id.trim().is_empty() {
        return Err(invalid("workspace version id must not be empty"));
    }
    if let Some(derived_from) = &version.provenance.derived_from
        && derived_from.trim().is_empty()
    {
        return Err(invalid(
            "workspace derived_from must not be empty when present",
        ));
    }

    let mut version = version;
    version.metadata.content_checksum = content_checksum(&version.content)?;
    Ok(version)
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::{
        WorkspaceArtifactId, WorkspaceArtifactVersion, WorkspaceScope, content_checksum,
        normalize_version,
    };
    use serde_json::json;

    #[test]
    fn ids_must_not_be_empty() {
        assert!(WorkspaceArtifactId::new("").is_err());
        assert!(WorkspaceScope::new("").is_err());
    }

    #[test]
    fn ids_reject_path_traversal() {
        assert!(WorkspaceArtifactId::new("../escape").is_err());
        assert!(WorkspaceScope::new("nested/path").is_err());
    }

    #[test]
    fn versions_preserve_payload() {
        let version = WorkspaceArtifactVersion::new("v1", json!({"kind": "note"})).expect("v1");
        assert_eq!(version.version_id, "v1");
        assert_eq!(version.content["kind"], json!("note"));
    }

    #[test]
    fn normalization_sets_checksum() {
        let version = normalize_version(
            WorkspaceArtifactVersion::new("v1", json!({"kind": "note"})).expect("v1"),
        )
        .expect("normalized");
        assert_eq!(
            version.metadata.content_checksum,
            content_checksum(&json!({"kind": "note"})).expect("checksum")
        );
    }
}
