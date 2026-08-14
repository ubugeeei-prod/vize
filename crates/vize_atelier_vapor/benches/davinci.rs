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

use criterion::{Criterion, criterion_group};
use davinci_harness::fixtures::{LADDER, template_block};
use davinci_harness::stage::bench_stage_with_metrics;
use vize_atelier_core::expr_parse_probe;
use vize_atelier_core::lane::transform;
use vize_atelier_core::options::TransformOptions;
use vize_atelier_core::parser::Parser;
use vize_atelier_vapor::{
    VaporCompilerOptions, compile_vapor, drop_ir_stack_safe, generate_vapor, transform_to_ir,
};
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
            let ir = window.measure(|| transform_to_ir(&allocator, &root, template));
            drop_ir_stack_safe(ir);
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
        drop_ir_stack_safe(ir);

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

criterion_group!(davinci_group, davinci);
davinci_harness::main!(davinci_group);
