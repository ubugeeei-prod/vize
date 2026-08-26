//! Davinci microbenches: the transform lane over the P0-2 fixture ladder.
//!
//! Run with: cargo bench -p vize_atelier_core --bench davinci
//!
//! `transform` mutates the AST in place and `RootNode` has no `Clone`, so
//! every iteration rebuilds its input (fresh arena + parse) as unmeasured
//! setup and only the `transform` call enters the metrics, via
//! `davinci_harness::stage`. The lane runs with `TransformOptions::default()`
//! - the backend-flavored variants are priced in the dom/vapor/ssr benches.

use criterion::{Criterion, criterion_group};
use davinci_harness::fixtures::{LADDER, template_block};
use davinci_harness::stage::bench_stage_with_metrics;
use vize_atelier_core::lane::transform;
use vize_atelier_core::options::TransformOptions;
use vize_atelier_core::parser::Parser;
use vize_s0::{Allocator, cstr};

fn davinci(criterion: &mut Criterion) {
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

criterion_group!(davinci_group, davinci);
davinci_harness::main!(davinci_group);
