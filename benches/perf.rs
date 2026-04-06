#[path = "../tests/perf_support/mod.rs"]
mod perf_support;

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use greentic_dw_providers_common::{
    PackId, ProviderCategory, sample_pack_manifest, sample_pack_manifest_cbor,
    validate_provider_extension_inline,
};

fn perf_pack_id() -> PackId {
    PackId::new("vendor.provider.perf").expect("valid benchmark pack id")
}

fn bench_provider_extension_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("provider_extension_validate");

    for provider_count in [1_usize, 4, 8, 16] {
        let extension = perf_support::provider_extension_fixture(provider_count);
        group.throughput(Throughput::Elements(provider_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(provider_count),
            &provider_count,
            |b, _| {
                b.iter(|| {
                    validate_provider_extension_inline(black_box(&extension))
                        .expect("benchmark extension should validate");
                })
            },
        );
    }

    group.finish();
}

fn bench_pack_manifest_cbor(c: &mut Criterion) {
    let pack_id = perf_pack_id();
    c.bench_function("pack_manifest_cbor", |b| {
        b.iter(|| {
            black_box(
                sample_pack_manifest_cbor(
                    black_box(pack_id.clone()),
                    ProviderCategory::Memory,
                    "shortterm",
                    "shortterm",
                )
                .expect("benchmark pack manifest should encode"),
            )
        })
    });
}

fn bench_pack_manifest_build(c: &mut Criterion) {
    let pack_id = perf_pack_id();
    c.bench_function("pack_manifest_build", |b| {
        b.iter(|| {
            black_box(
                sample_pack_manifest(
                    black_box(pack_id.clone()),
                    ProviderCategory::Memory,
                    "shortterm",
                    "shortterm",
                )
                .expect("benchmark pack manifest should build"),
            )
        })
    });
}

criterion_group!(
    benches,
    bench_provider_extension_validation,
    bench_pack_manifest_build,
    bench_pack_manifest_cbor
);
criterion_main!(benches);
