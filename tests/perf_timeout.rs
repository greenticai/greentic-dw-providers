use std::hint::black_box;
use std::time::{Duration, Instant};

use greentic_dw_providers_common::validate_provider_extension_inline;

mod perf_support;

#[allow(unexpected_cfgs)]
fn timeout_budget() -> Duration {
    if cfg!(coverage) {
        Duration::from_secs(4)
    } else {
        Duration::from_secs(2)
    }
}

#[test]
fn workload_should_finish_quickly() {
    let extension = perf_support::provider_extension_fixture(16);
    let start = Instant::now();

    for _ in 0..10_000 {
        validate_provider_extension_inline(black_box(&extension))
            .expect("perf extension should validate");
    }

    let elapsed = start.elapsed();
    let budget = timeout_budget();

    assert!(
        elapsed < budget,
        "workload too slow: {:?} (budget: {:?})",
        elapsed,
        budget
    );
}
