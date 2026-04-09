# Observer Basic Metrics

Basic metrics observer provider crate.

This backend now contains real Rust code:

- crate: `greentic-dw-observer-basic-metrics`
- shared contract crate: `greentic-dw-observer`
- provider type `dw.observer.basic-metrics`
- capability URI `cap://dw.observer.metrics`
- pack capability id `greentic.cap.observer.metrics`

The provider aggregates per-tenant counts and numeric totals by event kind and returns those
aggregates through a tenant-scoped metrics report.
