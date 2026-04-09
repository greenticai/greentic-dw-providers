#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Basic audit observer provider.

use greentic_dw_observer::{Observer, ObserverEvent, ObserverReport};
use greentic_dw_providers_common::{
    ObserverVariant, observer_pack_manifest, observer_provider_decl,
};
use greentic_types::{
    ErrorCode, GResult, GreenticError, PackId, PackManifest, ProviderDecl, TenantCtx,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

/// Basic audit observer that stores a per-tenant append-only event log.
#[derive(Default)]
pub struct BasicAuditObserver {
    entries: Mutex<BTreeMap<String, Vec<ObserverEvent>>>,
}

impl BasicAuditObserver {
    /// Creates an empty observer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the canonical provider declaration for this backend.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        observer_provider_decl(ObserverVariant::BasicAudit)
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.observer.basic-audit")?;
        observer_pack_manifest(pack_id, ObserverVariant::BasicAudit)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }

    /// Returns all recorded audit events for the given tenant.
    pub fn entries(&self, tenant: &TenantCtx) -> GResult<Vec<ObserverEvent>> {
        let guard = self.lock_entries()?;
        Ok(guard
            .get(tenant.tenant_id.as_str())
            .cloned()
            .unwrap_or_default())
    }

    fn lock_entries(&self) -> GResult<MutexGuard<'_, BTreeMap<String, Vec<ObserverEvent>>>> {
        self.entries
            .lock()
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }
}

impl Observer for BasicAuditObserver {
    fn observe(&self, tenant: &TenantCtx, event: ObserverEvent) -> GResult<()> {
        let mut guard = self.lock_entries()?;
        guard
            .entry(tenant.tenant_id.to_string())
            .or_default()
            .push(event);
        Ok(())
    }

    fn report(&self, tenant: &TenantCtx) -> GResult<ObserverReport> {
        let entries = self.entries(tenant)?;
        let mut event_counts = BTreeMap::new();
        let mut totals_by_kind = BTreeMap::new();

        for entry in &entries {
            *event_counts.entry(entry.kind.clone()).or_insert(0) += 1;
            *totals_by_kind.entry(entry.kind.clone()).or_insert(0.0) += entry.value;
        }

        Ok(ObserverReport {
            report_type: "audit".to_string(),
            tenant_id: tenant.tenant_id.to_string(),
            total_events: entries.len() as u64,
            event_counts,
            totals_by_kind,
            details: json!({
                "entries": entries,
            }),
        })
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::BasicAuditObserver;
    use greentic_dw_observer::{Observer, ObserverEvent};
    use greentic_types::{EnvId, TenantCtx, TenantId};

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-audit").expect("tenant id"),
        )
    }

    #[test]
    fn basic_audit_observer_records_entries() {
        let observer = BasicAuditObserver::new();
        let tenant = tenant();

        observer
            .observe(
                &tenant,
                ObserverEvent::new("task.started", "task started").expect("event"),
            )
            .expect("observe");

        let entries = observer.entries(&tenant).expect("entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].kind, "task.started");
    }

    #[test]
    fn basic_audit_observer_reports_counts() {
        let observer = BasicAuditObserver::new();
        let tenant = tenant();

        observer
            .observe(
                &tenant,
                ObserverEvent::new("task.started", "task started").expect("event"),
            )
            .expect("observe");
        observer
            .observe(
                &tenant,
                ObserverEvent::new("task.started", "task started").expect("event"),
            )
            .expect("observe");

        let report = observer.report(&tenant).expect("report");
        assert_eq!(report.report_type, "audit");
        assert_eq!(report.total_events, 2);
        assert_eq!(report.event_counts["task.started"], 2);
    }
}
