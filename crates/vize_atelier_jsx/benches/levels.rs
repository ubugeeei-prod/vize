//! Instruction rows for the JSX/TSX compile entry (#6885).
//!
//! Each row measures one public `compile_jsx` call over the same module,
//! from the oxc parse to the emitted module, in one configuration: native
//! VDOM, Babel-compatible VDOM and native Vapor. SSR refuses expression-bodied
//! VDOM components today, so its row lands with the SSR slice. The fixture's
//! roots cover the constructs the shared L2 lowering admits today and the ones
//! its next slices move (`&&` over values, non-element branches, `_`
//! modifiers, mixed text and slot return values, the Babel dialect), so each
//! slice shows up in these rows. Every row must compile the fixture without
//! errors. The arena is created before the window and the output is dropped
//! after it.
//!
//! Run with: cargo bench -p vize_atelier_jsx --bench levels
#![expect(clippy::expect_used, reason = "benchmarks abort on fixture errors")]

use criterion::{Criterion, criterion_group};
use davinci_harness::stage::bench_stage_with_metrics;
use vize_atelier_jsx::{JsxCompatMode, JsxCompileConfig, JsxLang, JsxOutputMode, compile_jsx};
use vize_l0::{Allocator, cstr};

const FIXTURE: &str = "crates/vize_atelier_jsx/benches/fixtures/compile-slices.jsx";
const SOURCE: &str = include_str!("fixtures/compile-slices.jsx");

fn compile_row(criterion: &mut Criterion, id: &str, config: &JsxCompileConfig) {
    bench_stage_with_metrics(criterion, id, FIXTURE, |window| {
        let allocator = Allocator::new();
        let output = window.measure(|| compile_jsx(&allocator, SOURCE, JsxLang::Jsx, config));
        assert!(
            !output.has_errors(),
            "{id}: the fixture must compile cleanly: {:?}",
            output.diagnostics
        );
        output.components.len()
    });
}

fn levels(criterion: &mut Criterion) {
    compile_row(
        criterion,
        &cstr!("atelier_jsx_compile_vdom_slices"),
        &JsxCompileConfig::default(),
    );
    compile_row(
        criterion,
        &cstr!("atelier_jsx_compile_babel_slices"),
        &JsxCompileConfig {
            compat: JsxCompatMode::Babel,
            ..JsxCompileConfig::default()
        },
    );
    compile_row(
        criterion,
        &cstr!("atelier_jsx_compile_vapor_slices"),
        &JsxCompileConfig {
            default_mode: JsxOutputMode::Vapor,
            ..JsxCompileConfig::default()
        },
    );
}

criterion_group!(levels_group, levels);
davinci_harness::main!(levels_group);
