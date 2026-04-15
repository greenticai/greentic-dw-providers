#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Provenance-aware retrieval context provider.

use greentic_dw_context::{
    ContextFragment, ContextPackage, ContextProvider, ContextRequest, ContextSourceKind,
};
use greentic_dw_planning::PlanDocument;
use greentic_dw_workspace::{WorkspaceArtifactId, WorkspaceProvider, WorkspaceScope};
use greentic_types::{GResult, TenantCtx};
use serde_json::Value;

/// A retrieval source reference.
#[derive(Clone, Debug, PartialEq)]
pub enum RetrievalRef {
    /// A workspace artifact ref scoped by workspace scope and artifact id.
    Workspace {
        /// Workspace scope.
        scope: WorkspaceScope,
        /// Artifact id.
        artifact_id: WorkspaceArtifactId,
        /// Ranking score.
        score: f64,
    },
    /// A plan-step reference from a materialized plan.
    PlanStep {
        /// Plan document to read from.
        plan: PlanDocument,
        /// Step id to materialize.
        step_id: String,
        /// Ranking score.
        score: f64,
    },
    /// An opaque memory reference.
    MemoryRef {
        /// Stable memory reference string.
        reference: String,
        /// Materialized content.
        content: Value,
        /// Ranking score.
        score: f64,
    },
}

/// Retrieval provider configuration.
pub struct RetrievalContextProvider<W> {
    workspace: W,
    refs: Vec<RetrievalRef>,
}

impl<W> RetrievalContextProvider<W> {
    /// Creates a retrieval provider from workspace access and configured refs.
    #[must_use]
    pub fn new(workspace: W, refs: Vec<RetrievalRef>) -> Self {
        Self { workspace, refs }
    }
}

impl<W: WorkspaceProvider> ContextProvider for RetrievalContextProvider<W> {
    fn assemble(&self, tenant: &TenantCtx, _request: ContextRequest) -> GResult<ContextPackage> {
        let mut fragments = Vec::new();
        for source in &self.refs {
            match source {
                RetrievalRef::Workspace {
                    scope,
                    artifact_id,
                    score,
                } => {
                    let history = self.workspace.load_history(tenant, scope, artifact_id)?;
                    if let Some(version) = history.last() {
                        fragments.push(
                            ContextFragment::new(
                                format!("workspace:{}:{}", scope.0, artifact_id.0),
                                version.content.clone(),
                            )?
                            .with_source_kind(ContextSourceKind::Workspace)
                            .with_provenance_ref(format!(
                                "workspace/{}/{}/{}",
                                scope.0, artifact_id.0, version.version_id
                            ))
                            .with_score(*score),
                        );
                    }
                }
                RetrievalRef::PlanStep {
                    plan,
                    step_id,
                    score,
                } => {
                    if let Some(step) = plan.steps.iter().find(|step| &step.id == step_id) {
                        fragments.push(
                            ContextFragment::new(
                                format!("plan-step:{step_id}"),
                                step.payload.clone(),
                            )?
                            .with_source_kind(ContextSourceKind::PlanStep)
                            .with_provenance_ref(format!("plan/{}/{}", plan.plan_id, step.id))
                            .with_score(*score),
                        );
                    }
                }
                RetrievalRef::MemoryRef {
                    reference,
                    content,
                    score,
                } => {
                    fragments.push(
                        ContextFragment::new(format!("memory:{reference}"), content.clone())?
                            .with_source_kind(ContextSourceKind::MemoryRef)
                            .with_provenance_ref(format!("memory/{reference}"))
                            .with_score(*score),
                    );
                }
            }
        }

        fragments.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(ContextPackage::empty()
            .with_metadata(serde_json::json!({
                "provider": "retrieval",
                "fragment_count": fragments.len()
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
