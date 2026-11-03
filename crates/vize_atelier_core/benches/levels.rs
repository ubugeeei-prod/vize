//! Levels microbenches: the transform lane over the P0-2 fixture ladder.
//!
//! Run with: cargo bench -p vize_atelier_core --bench levels
//!
//! `transform` mutates the AST in place and `RootNode` has no `Clone`, so
//! every iteration rebuilds its input (fresh arena + parse) as unmeasured
//! setup and only the `transform` call enters the metrics, via
//! `level_harness::stage`. The lane runs with `TransformOptions::default()`
//! - the backend-flavored variants are priced in the dom/vapor/ssr benches.
#![expect(clippy::expect_used, reason = "benchmarks abort on fixture errors")]

use criterion::{Criterion, criterion_group};
use level_harness::fixtures::{LADDER, template_block};
use level_harness::stage::bench_stage_with_metrics;
use vize_atelier_core::lane::transform;
use vize_atelier_core::options::TransformOptions;
use vize_atelier_core::parser::Parser;
use vize_l0::{Allocator, cstr};

fn levels(criterion: &mut Criterion) {
    for fixture in &LADDER {
        let template =
            template_block(fixture.source).expect("every ladder fixture has a template block");

        let bench_id = cstr!("atelier_core_transform_{}", fixture.name);
        bench_stage_with_metrics(criterion, &bench_id, fixture.relative_path, |window| {
            let allocator = Allocator::new();
            let (mut root, _errors) = Parser::new(&allocator, template).parse();
            window.measure(|| transform(&allocator, &mut root, TransformOptions::default(), None))
        });
    }
}

criterion_group!(level_group, levels);
level_harness::main!(level_group);
