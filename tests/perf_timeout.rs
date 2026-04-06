use std::hint::black_box;
use std::time::{Duration, Instant};

use greentic_dw_providers_common::validate_provider_extension_inline;

mod perf_support;

#[test]
fn workload_should_finish_quickly() {
    let extension = perf_support::provider_extension_fixture(16);
    let start = Instant::now();

    for _ in 0..10_000 {
        validate_provider_extension_inline(black_box(&extension))
            .expect("perf extension should validate");
    }

    let elapsed = start.elapsed();

    assert!(
        elapsed < Duration::from_secs(2),
        "workload too slow: {:?}",
        elapsed
    );
}
