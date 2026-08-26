//! Davinci microbenches: DOM backend transform vs codegen, split (P0-3).
//!
//! Run with: cargo bench -p vize_atelier_dom --bench davinci
//!
//! The public `compile_template` entry is fused, but the DOM pipeline is the
//! core lane's `transform` (with DOM-flavored options) followed by
//! `generate_with_sections`, so the stages are timed separately here:
//! transform rebuilds its input per iteration (stage window, setup
//! unmeasured); codegen reads the transformed AST immutably and allocates no
//! arena data, so it loops over one prepared AST. A fused case pins the
//! end-to-end number the split must add up to (parse included).
//!
//! After all cases, the expression re-parse counter
//! (`vize_atelier_core::expr_parse_probe`) is sampled around one fused
//! compile per fixture - the `davinci.expr.parses` baseline recorded in
//! `davinci-road/plan/expr-reparse-baseline.md`.

use criterion::{Criterion, criterion_group};
use davinci_harness::fixtures::{LADDER, template_block};
use davinci_harness::stage::bench_stage_with_metrics;
use vize_atelier_core::codegen::generate_with_sections;
use vize_atelier_core::expr_parse_probe;
use vize_atelier_core::lane::transform;
use vize_atelier_core::options::{CodegenOptions, TransformOptions};
use vize_atelier_core::parser::Parser;
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_s0::{Allocator, cstr};

fn dom_transform_options(dom: &DomCompilerOptions) -> TransformOptions {
    TransformOptions {
        prefix_identifiers: dom.prefix_identifiers,
        hoist_static: dom.hoist_static,
        cache_handlers: dom.cache_handlers,
        ..TransformOptions::default()
    }
}

fn davinci(criterion: &mut Criterion) {
    let dom = DomCompilerOptions::default();

    for fixture in &LADDER {
        let template =
            template_block(fixture.source).expect("every ladder fixture has a template block");

        let transform_id = cstr!("atelier_dom_transform_{}", fixture.name);
        bench_stage_with_metrics(criterion, &transform_id, fixture.relative_path, |window| {
            let allocator = Allocator::new();
            let (mut root, _errors) = Parser::new(&allocator, template).parse();
            window.measure(|| transform(&allocator, &mut root, dom_transform_options(&dom), None))
        });

        let allocator = Allocator::new();
        let (mut root, _errors) = Parser::new(&allocator, template).parse();
        transform(&allocator, &mut root, dom_transform_options(&dom), None);
        let root = root;
        let codegen_id = cstr!("atelier_dom_codegen_{}", fixture.name);
        davinci_harness::bench_with_metrics(criterion, &codegen_id, fixture.relative_path, || {
            generate_with_sections(
                &root,
                CodegenOptions {
                    mode: dom.mode,
                    ..CodegenOptions::default()
                },
            )
        });

        let fused_id = cstr!("atelier_dom_compile_{}", fixture.name);
        davinci_harness::bench_with_metrics(criterion, &fused_id, fixture.relative_path, || {
            let allocator = Allocator::new();
            let (root, errors, result) =
                compile_template_with_options(&allocator, template, dom.clone());
            // The AST borrows the iteration-local arena; return owned facts.
            (root.children.len(), errors.len(), result.code.len())
        });
    }

    for fixture in &LADDER {
        let template =
            template_block(fixture.source).expect("every ladder fixture has a template block");
        let allocator = Allocator::new();
        let before = expr_parse_probe::expr_parse_count();
        let _compiled = compile_template_with_options(&allocator, template, dom.clone());
        let parses = expr_parse_probe::expr_parse_count() - before;
        eprintln!("davinci.expr.parses dom {} {parses}", fixture.name);
    }
}

criterion_group!(davinci_group, davinci);
davinci_harness::main!(davinci_group);
