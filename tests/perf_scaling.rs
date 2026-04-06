use std::hint::black_box;
use std::time::Instant;

use greentic_dw_providers_common::validate_provider_extension_inline;

mod perf_support;

fn run_workload(threads: usize) -> f64 {
    let extension = perf_support::provider_extension_fixture(16);
    let start = Instant::now();

    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let extension = extension.clone();
            std::thread::spawn(move || {
                for _ in 0..500 {
                    validate_provider_extension_inline(black_box(&extension))
                        .expect("perf extension should validate");
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("perf worker should not panic");
    }

    start.elapsed().as_secs_f64() / threads as f64
}

#[test]
fn scaling_should_not_degrade_badly() {
    let t1 = run_workload(1);
    let t4 = run_workload(4);
    let t8 = run_workload(8);

    assert!(
        t4 <= t1 * 1.6,
        "4-thread per-worker time regressed: t1={t1:?}, t4={t4:?}"
    );
    assert!(
        t8 <= t4 * 1.6,
        "8-thread per-worker time regressed: t4={t4:?}, t8={t8:?}"
    );
}
