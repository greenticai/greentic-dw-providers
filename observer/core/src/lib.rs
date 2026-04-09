#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::unwrap_used, clippy::expect_used)]

//! Shared observer contract and event/report models.

use greentic_types::{ErrorCode, GResult, GreenticError, TenantCtx};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Event observed by an observer provider.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObserverEvent {
    /// Stable event kind used for routing and aggregation.
    pub kind: String,
    /// Human-readable message for audit and diagnostics.
    pub message: String,
    /// Numeric value associated with the event.
    pub value: f64,
    /// Event timestamp in unix milliseconds.
    pub timestamp_ms: u64,
    /// Free-form attributes carried with the event.
    pub attributes: BTreeMap<String, String>,
    /// Optional JSON payload attached to the event.
    pub payload: Option<Value>,
}

impl ObserverEvent {
    /// Creates a new event stamped with the current wall-clock time.
    pub fn new(kind: impl Into<String>, message: impl Into<String>) -> GResult<Self> {
        let kind = kind.into();
        if kind.trim().is_empty() {
            return Err(invalid("observer event kind must not be empty"));
        }

        Ok(Self {
            kind,
            message: message.into(),
            value: 1.0,
            timestamp_ms: now_unix_ms()?,
            attributes: BTreeMap::new(),
            payload: None,
        })
    }

    /// Sets a numeric value for the event.
    #[must_use]
    pub fn with_value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    /// Adds an attribute to the event.
    #[must_use]
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Attaches a JSON payload to the event.
    #[must_use]
    pub fn with_payload(mut self, payload: Value) -> Self {
        self.payload = Some(payload);
        self
    }
}

/// Report produced by an observer provider.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObserverReport {
    /// Stable report kind.
    pub report_type: String,
    /// Tenant id this report is scoped to.
    pub tenant_id: String,
    /// Total number of observed events.
    pub total_events: u64,
    /// Count by event kind.
    pub event_counts: BTreeMap<String, u64>,
    /// Numeric totals by event kind.
    pub totals_by_kind: BTreeMap<String, f64>,
    /// Provider-specific details.
    pub details: Value,
}

/// Contract implemented by observer providers.
pub trait Observer: Send + Sync {
    /// Records an event.
    fn observe(&self, tenant: &TenantCtx, event: ObserverEvent) -> GResult<()>;

    /// Returns a report for the current tenant state.
    fn report(&self, tenant: &TenantCtx) -> GResult<ObserverReport>;
}

fn invalid(message: impl Into<String>) -> GreenticError {
    GreenticError::new(ErrorCode::InvalidInput, message)
}

fn now_unix_ms() -> GResult<u64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| GreenticError::new(ErrorCode::Internal, err.to_string()))?;
    Ok(duration.as_millis() as u64)
}

#[cfg(test)]
mod tests {
    use super::ObserverEvent;

    #[test]
    fn observer_event_rejects_empty_kind() {
        let err = match ObserverEvent::new("   ", "message") {
            Ok(_) => panic!("kind should be invalid"),
            Err(err) => err,
        };
        assert!(err.to_string().contains("kind"));
    }
}
