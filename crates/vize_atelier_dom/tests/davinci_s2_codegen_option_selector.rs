//! P2-11 witness: DOM-irrelevant codegen options keep the S2 production lane
//! selected after the compatibility no-op is audited.

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
fn optimize_imports_uses_s2_after_legacy_noop_audit() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    let source = r#"<button @click="go">{{ label }}</button>"#;
    let selected_default = compile(source, CodegenOptions::default());

    profiler.enable();
    let selected_optimized = compile(
        source,
        CodegenOptions {
            optimize_imports: true,
            ..Default::default()
        },
    );
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();

    assert_eq!(selected_optimized.preamble, selected_default.preamble);
    assert_eq!(selected_optimized.code, selected_default.code);
    assert_eq!(selected_optimized.sections, selected_default.sections);
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        Some(1),
        "DOM-irrelevant optimize_imports must not disarm the S2 production selector"
    );
}

struct Compiled {
    preamble: String,
    code: String,
    sections: Option<vize_atelier_core::CodegenSections>,
}

fn compile(source: &str, codegen: CodegenOptions) -> Compiled {
    let allocator = Allocator::new();
    let (_, errors, result) =
        compile_template_with_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options(
            &allocator,
            source,
            DomCompilerOptions::default(),
            TemplateSyntaxMode::Standard,
            None,
            codegen,
        );

    assert!(errors.is_empty(), "compile errors: {errors:?}");
    Compiled {
        preamble: result.result.preamble.to_string(),
        code: result.result.code.to_string(),
        sections: result.sections,
    }
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
