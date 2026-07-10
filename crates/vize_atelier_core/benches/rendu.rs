//! Rendu construction benchmarks.
//!
//! Guards the #1634 performance rule: walking transformed Relief as Rendu
//! operations must stay cheap and allocation-free. A regression here (for
//! example an owned render-tree rebuild on the hot path) should show up in the
//! PR benchmark budget rather than being discovered after the fact.
#![allow(deprecated)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use vize_atelier_core::{Allocator, TransformOptions, parse, transform};
use vize_rendu::summarize_rendu_ops;

const FLAT: &str = "<div><span>a</span><span>b</span><span>c</span><p>{{ msg }}</p></div>";
const NESTED: &str = "<ul><li><a><span><b><i>{{ deep }}</i></b></span></a></li></ul>";
const CONTROL_FLOW: &str =
    "<div v-if=\"ok\"><p v-for=\"x in items\">{{ x }}</p></div><p v-else>no</p>";

fn bench_walk(c: &mut Criterion, name: &str, source: &str) {
    c.bench_function(name, |b| {
        b.iter(|| {
            let allocator = Allocator::default();
            let bump = allocator.as_bump();
            let (mut root, _errors) = parse(bump, black_box(source));
            let _ = transform(bump, &mut root, TransformOptions::default(), None);
            black_box(summarize_rendu_ops(&root))
        });
    });
}

fn benchmark_rendu_walk(c: &mut Criterion) {
    bench_walk(c, "rendu_walk_flat", FLAT);
    bench_walk(c, "rendu_walk_nested", NESTED);
    bench_walk(c, "rendu_walk_control_flow", CONTROL_FLOW);
}

criterion_group!(benches, benchmark_rendu_walk);
criterion_main!(benches);
