//! Davinci microbenches: the Vapor pipeline split into its three stages.
//!
//! Run with: cargo bench -p vize_atelier_vapor --bench davinci
//!
//! Today's Vapor compile runs the full VDOM transform lane and then lowers
//! from the surface AST, discarding the lane's codegen-node output
//! (`compile.rs` lines 163-175 - the run-then-discard double transform).
//! These benches pin each stage as its own number:
//!
//! - `transform`: the VDOM lane with `vapor: true` - the cost the double
//!   transform pays today, priced so phase 3 can show it disappearing.
//! - `lower`: `transform_to_ir` from a transformed AST. Production shares
//!   one arena across parse/transform/lower, so the iteration rebuilds that
//!   arena as unmeasured setup and only the lowering call is measured.
//! - `generate`: `generate_vapor` over one prepared IR - pure function into
//!   owned output, so it loops without arena growth.
//!
//! A fused `compile_vapor` case pins the end-to-end number, and the
//! expression re-parse counter is sampled around one fused compile per
//! fixture for the `davinci.expr.parses` baseline.

#![expect(
    clippy::expect_used,
    reason = "benchmark setup aborts on a broken fixture"
)]

use criterion::{Criterion, criterion_group};
use davinci_harness::fixtures::{LADDER, template_block};
use davinci_harness::stage::bench_stage_with_metrics;
use vize_atelier_core::expr_parse_probe;
use vize_atelier_core::lane::transform;
use vize_atelier_core::options::TransformOptions;
use vize_atelier_core::parser::Parser;
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor, generate_vapor, transform_to_ir};
use vize_carton::{Allocator, cstr};

fn vapor_transform_options() -> TransformOptions {
    TransformOptions {
        vapor: true,
        ..TransformOptions::default()
    }
}

fn davinci(criterion: &mut Criterion) {
    for fixture in &LADDER {
        let template =
            template_block(fixture.source).expect("every ladder fixture has a template block");

        let transform_id = cstr!("atelier_vapor_transform_{}", fixture.name);
        bench_stage_with_metrics(criterion, &transform_id, fixture.relative_path, |window| {
            let allocator = Allocator::new();
            let (mut root, _errors) = Parser::new(&allocator, template).parse();
            window.measure(|| transform(&allocator, &mut root, vapor_transform_options(), None))
        });

        let lower_id = cstr!("atelier_vapor_lower_{}", fixture.name);
        bench_stage_with_metrics(criterion, &lower_id, fixture.relative_path, |window| {
            let allocator = Allocator::new();
            let (mut root, _errors) = Parser::new(&allocator, template).parse();
            transform(&allocator, &mut root, vapor_transform_options(), None);
            let _ir = window.measure(|| transform_to_ir(&allocator, &root, template));
        });

        let allocator = Allocator::new();
        let (mut root, _errors) = Parser::new(&allocator, template).parse();
        transform(&allocator, &mut root, vapor_transform_options(), None);
        let root = root;
        let ir = transform_to_ir(&allocator, &root, template);
        let generate_id = cstr!("atelier_vapor_generate_{}", fixture.name);
        davinci_harness::bench_with_metrics(criterion, &generate_id, fixture.relative_path, || {
            generate_vapor(&ir, None)
        });

        let fused_id = cstr!("atelier_vapor_compile_{}", fixture.name);
        davinci_harness::bench_with_metrics(criterion, &fused_id, fixture.relative_path, || {
            let allocator = Allocator::new();
            compile_vapor(&allocator, template, VaporCompilerOptions::default())
        });
    }

    for fixture in &LADDER {
        let template =
            template_block(fixture.source).expect("every ladder fixture has a template block");
        let allocator = Allocator::new();
        let before = expr_parse_probe::expr_parse_count();
        let _compiled = compile_vapor(&allocator, template, VaporCompilerOptions::default());
        let parses = expr_parse_probe::expr_parse_count() - before;
        eprintln!("davinci.expr.parses vapor {} {parses}", fixture.name);
    }
}

/// Compare the admitted production route and explicitly retained legacy route
/// in the same process. The explicit retained-lane selector changes no binding.
fn native_pair(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("vapor_native_pair");
    for (name, source) in [
        (
            "text_runs",
            "<main>Hello {{ name }}!<span>{{ first }} / {{ last }}</span><button @click=\"save\">Save {{ count }}</button></main>",
        ),
        (
            "events",
            "<main @keydown=\"save\"><button @click.stop=\"save\" @keydown.enter.stop=\"save\">{{ label }}</button><input @focus=\"save\" @change.once=\"save\"></main>",
        ),
        (
            "expressions",
            "<main class=\"shell\" :class=\"{ dense, [theme]: true }\"><button @click=\"count++\" :title=\"'n=' + count\">{{ count * 2 }} / {{ label.toUpperCase() }}</button><div v-show=\"open && ready\" v-text=\"items.map(i => i.name).join(', ')\"></div></main>",
        ),
        (
            "components",
            "<main><Counter :label=\"title\" :step=\"2\" @bump=\"total += $event\"><b>{{ total }}</b></Counter><slot name=\"aside\" :n=\"total\"><i>none</i></slot></main>",
        ),
        (
            "templates",
            "<main><template v-if=\"open\"><header>{{ title }}</header><section>{{ lead }}</section></template><ul><template v-for=\"row in rows\" :key=\"row.id\"><li>{{ row.label }}</li><li v-if=\"row.note\">{{ row.note }}</li></template></ul></main>",
        ),
        (
            "spreads",
            "<main class=\"shell\"><section id=\"card\" class=\"card\" v-bind=\"attrs\" :title=\"title\"><b v-bind=\"badge\">{{ count }}</b></section><Panel v-bind=\"panel\" v-on=\"{ close: save }\" :size=\"size\" /><button v-on=\"handlers\">go</button></main>",
        ),
        (
            "control_flow",
            "<main><section v-if=\"open\"><b>{{ title }}</b><span v-for=\"row in rows\" :key=\"row.id\" :title=\"row.title\">{{ row.label }}</span></section><i v-else>closed</i><ul><li v-for=\"(cell, i) in cells\" @click=\"save\">{{ i }}: {{ cell }}</li></ul></main>",
        ),
    ] {
        for (lane, legacy) in [("s3", false), ("legacy", true)] {
            group.bench_function(criterion::BenchmarkId::new(name, lane), |bencher| {
                bencher.iter(|| {
                    let allocator = Allocator::new();
                    std::hint::black_box(compile_vapor(
                        &allocator,
                        source,
                        VaporCompilerOptions {
                            prefix_identifiers: true,
                            davinci_retained_lane: legacy,
                            ..Default::default()
                        },
                    ))
                });
            });
        }
    }
    group.finish();
}

criterion_group!(davinci_group, davinci, native_pair);
davinci_harness::main!(davinci_group);
