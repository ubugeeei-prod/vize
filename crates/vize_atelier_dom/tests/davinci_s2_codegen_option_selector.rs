//! P2-11 witness: backend-only codegen options stay on the compatibility lane
//! until the DOM S2 selector owns their semantics.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use vize_atelier_core::options::{CodegenOptions, TemplateSyntaxMode};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_template_with_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
};
use vize_s0::Allocator;
use vize_s0::profiler::{CounterSummary, global_profiler};

#[test]
fn optimize_imports_stays_on_compatibility_when_profiled() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();
    profiler.enable();

    let allocator = Allocator::new();
    let (_, errors, result) =
        compile_template_with_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options(
            &allocator,
            r#"<button @click="go">{{ label }}</button>"#,
            DomCompilerOptions::default(),
            TemplateSyntaxMode::Standard,
            None,
            CodegenOptions {
                optimize_imports: true,
                ..Default::default()
            },
        );
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();

    assert!(errors.is_empty(), "compile errors: {errors:?}");
    assert!(
        result.sections.is_some(),
        "unsupported backend-only codegen options must keep compatibility sections"
    );
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        None,
        "unsupported backend-only codegen options must not enter the S2 production selector"
    );
}

fn counter_total(counters: &CounterSummary, name: &str) -> Option<u64> {
    counters
        .entries
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.total)
}

fn lock_profiler() -> std::sync::MutexGuard<'static, ()> {
    static PROFILER_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    PROFILER_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
