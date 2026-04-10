#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Basic metrics observer provider.

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

#[derive(Clone, Debug, Default)]
struct TenantMetrics {
    total_events: u64,
    event_counts: BTreeMap<String, u64>,
    totals_by_kind: BTreeMap<String, f64>,
    attribute_counts: BTreeMap<String, u64>,
}

/// Basic metrics observer that aggregates counts and totals per tenant.
#[derive(Default)]
pub struct BasicMetricsObserver {
    metrics: Mutex<BTreeMap<String, TenantMetrics>>,
}

impl BasicMetricsObserver {
    /// Creates an empty observer.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the canonical provider declaration for this backend.
    #[must_use]
    pub fn provider_decl() -> ProviderDecl {
        observer_provider_decl(ObserverVariant::BasicMetrics)
    }

    /// Returns the canonical pack manifest for this backend.
    pub fn pack_manifest() -> GResult<PackManifest> {
        let pack_id = PackId::new("greentic.dw.providers.observer.basic-metrics")?;
        observer_pack_manifest(pack_id, ObserverVariant::BasicMetrics)
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }

    fn lock_metrics(&self) -> GResult<MutexGuard<'_, BTreeMap<String, TenantMetrics>>> {
        self.metrics
            .lock()
            .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))
    }
}

impl Observer for BasicMetricsObserver {
    fn observe(&self, tenant: &TenantCtx, event: ObserverEvent) -> GResult<()> {
        let mut guard = self.lock_metrics()?;
        let entry = guard.entry(tenant.tenant_id.to_string()).or_default();
        entry.total_events += 1;
        *entry.event_counts.entry(event.kind.clone()).or_insert(0) += 1;
        *entry.totals_by_kind.entry(event.kind).or_insert(0.0) += event.value;

        for key in event.attributes.keys() {
            *entry.attribute_counts.entry(key.clone()).or_insert(0) += 1;
        }
        Ok(())
    }

    fn report(&self, tenant: &TenantCtx) -> GResult<ObserverReport> {
        let guard = self.lock_metrics()?;
        let metrics = guard
            .get(tenant.tenant_id.as_str())
            .cloned()
            .unwrap_or_default();

        Ok(ObserverReport {
            report_type: "metrics".to_string(),
            tenant_id: tenant.tenant_id.to_string(),
            total_events: metrics.total_events,
            event_counts: metrics.event_counts.clone(),
            totals_by_kind: metrics.totals_by_kind.clone(),
            details: json!({
                "attribute_counts": metrics.attribute_counts,
            }),
        })
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::BasicMetricsObserver;
    use greentic_dw_observer::{Observer, ObserverEvent};
    use greentic_types::{EnvId, TenantCtx, TenantId};

    fn tenant() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("env id"),
            TenantId::try_from("tenant-metrics").expect("tenant id"),
        )
    }

    #[test]
    fn basic_metrics_observer_aggregates_events() {
        let observer = BasicMetricsObserver::new();
        let tenant = tenant();

        observer
            .observe(
                &tenant,
                ObserverEvent::new("latency", "latency sample")
                    .expect("event")
                    .with_value(12.5)
                    .with_attribute("route", "/health"),
            )
            .expect("observe");
        observer
            .observe(
                &tenant,
                ObserverEvent::new("latency", "latency sample")
                    .expect("event")
                    .with_value(7.5),
            )
            .expect("observe");

        let report = observer.report(&tenant).expect("report");
        assert_eq!(report.report_type, "metrics");
        assert_eq!(report.total_events, 2);
        assert_eq!(report.event_counts["latency"], 2);
        assert_eq!(report.totals_by_kind["latency"], 20.0);
    }
}
