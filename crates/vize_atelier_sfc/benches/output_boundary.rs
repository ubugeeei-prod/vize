//! Compare the old and shared-parse NAPI output paths on identical emitted code.
//! Run: cargo bench -p vize_atelier_sfc --bench output_boundary
#![expect(clippy::expect_used, reason = "benchmarks abort on fixture errors")]
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions,
    compile_script::typescript::ensure_javascript_output,
    compile_sfc,
    module_shape::{analyze_module_shape, finalize_module_output},
    parse_sfc,
};

fn output_boundary(c: &mut Criterion) {
    let mut group = c.benchmark_group("output_boundary");
    for (name, source) in [
        (
            "small",
            include_str!("../../../tools/benchmarks/crates/davinci_harness/fixtures/small.vue"),
        ),
        (
            "medium",
            include_str!("../../../tools/benchmarks/crates/davinci_harness/fixtures/medium.vue"),
        ),
        (
            "large",
            include_str!("../../../tools/benchmarks/crates/davinci_harness/fixtures/large.vue"),
        ),
    ] {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse fixture");
        let code = compile_sfc(&descriptor, SfcCompileOptions::default())
            .expect("compile fixture")
            .code;
        group.throughput(Throughput::Bytes(code.len() as u64));
        group.bench_function(BenchmarkId::new("separate_parses", name), |b| {
            b.iter_batched(
                || code.clone(),
                |code| {
                    let code = ensure_javascript_output(black_box(code));
                    let shape = analyze_module_shape(&code);
                    black_box((code, shape))
                },
                BatchSize::SmallInput,
            );
        });
        group.bench_function(BenchmarkId::new("shared_parse", name), |b| {
            b.iter_batched(
                || code.clone(),
                |code| black_box(finalize_module_output(black_box(code), false)),
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(benches, output_boundary);
criterion_main!(benches);
