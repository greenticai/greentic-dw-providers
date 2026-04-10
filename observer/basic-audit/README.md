# Observer Basic Audit

Basic audit observer provider crate.

This backend now contains real Rust code:

- crate: `greentic-dw-observer-basic-audit`
- shared contract crate: `greentic-dw-observer`
- provider type `dw.observer.basic-audit`
- capability URI `cap://dw.observer.audit`
- pack capability id `greentic.cap.observer.audit`

The provider keeps an append-only in-process audit log per tenant and can emit a tenant-scoped
audit report with recorded entries and counts by event kind.
