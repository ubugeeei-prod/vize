//! Davinci microbenches: SSR codegen over the P0-2 fixture ladder.
//!
//! Run with: cargo bench -p vize_atelier_ssr --bench davinci
//!
//! `SsrCodegenContext` borrows the compile arena (production shares one
//! arena across parse/transform/codegen), so the iteration rebuilds arena +
//! parse + SSR-flavored transform as unmeasured setup and measures only the
//! context build + `generate` call - matching `compile_ssr`'s own codegen
//! stage without letting a shared arena grow across iterations. A fused
//! `compile_ssr` case pins the end-to-end number, and the expression
//! re-parse counter is sampled around one fused compile per fixture for the
//! `davinci.expr.parses` baseline.

use criterion::{Criterion, criterion_group};
use davinci_harness::fixtures::{LADDER, template_block};
use davinci_harness::stage::bench_stage_with_metrics;
use vize_atelier_core::expr_parse_probe;
use vize_atelier_core::lane::transform;
use vize_atelier_core::options::TransformOptions;
use vize_atelier_core::parser::Parser;
use vize_atelier_ssr::{SsrCodegenContext, SsrCompilerOptions, compile_ssr};
use vize_s0::{Allocator, cstr};

fn ssr_transform_options() -> TransformOptions {
    TransformOptions {
        ssr: true,
        prefix_identifiers: true,
        hoist_static: false,
        cache_handlers: false,
        ..TransformOptions::default()
    }
}

fn davinci(criterion: &mut Criterion) {
    let options = SsrCompilerOptions::default();

    for fixture in &LADDER {
        let template =
            template_block(fixture.source).expect("every ladder fixture has a template block");

        let codegen_id = cstr!("atelier_ssr_codegen_{}", fixture.name);
        bench_stage_with_metrics(criterion, &codegen_id, fixture.relative_path, |window| {
            let allocator = Allocator::new();
            let (mut root, _errors) = Parser::new(&allocator, template).parse();
            transform(&allocator, &mut root, ssr_transform_options(), None);
            window
                .measure(|| SsrCodegenContext::new(&allocator, &options, template).generate(&root))
        });

        let fused_id = cstr!("atelier_ssr_compile_{}", fixture.name);
        davinci_harness::bench_with_metrics(criterion, &fused_id, fixture.relative_path, || {
            let allocator = Allocator::new();
            let (root, errors, result) = compile_ssr(&allocator, template);
            // The AST borrows the iteration-local arena; return owned facts.
            (root.children.len(), errors.len(), result.code.len())
        });
    }

    for fixture in &LADDER {
        let template =
            template_block(fixture.source).expect("every ladder fixture has a template block");
        let allocator = Allocator::new();
        let before = expr_parse_probe::expr_parse_count();
        let _compiled = compile_ssr(&allocator, template);
        let parses = expr_parse_probe::expr_parse_count() - before;
        eprintln!("davinci.expr.parses ssr {} {parses}", fixture.name);
    }
}

criterion_group!(davinci_group, davinci);
davinci_harness::main!(davinci_group);
