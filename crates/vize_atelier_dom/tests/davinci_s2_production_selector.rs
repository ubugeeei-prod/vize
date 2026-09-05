//! P2-11 witness: supported source-map-free DOM compiles use the S2 production
//! selector even when profiling is disabled.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use vize_atelier_core::options::{CodegenOptions, CustomElementMatcher, TemplateSyntaxMode};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
    compile_template_with_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
};
use vize_s0::Allocator;
use vize_s0::profiler::{CounterSummary, global_profiler};

#[test]
fn source_map_free_dom_compile_uses_s2_without_profiler() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    let source = r#"<button @click="go">{{ label }}</button>"#;
    let compat = compile(
        source,
        DomCompilerOptions {
            source_map: true,
            ..Default::default()
        },
    );
    assert!(
        compat.sections.is_some(),
        "source-map compiles must retain compatibility sections"
    );

    let selected = compile(source, DomCompilerOptions::default());
    let counters = profiler.counter_summary();

    assert_eq!(selected.preamble, compat.preamble);
    assert_eq!(selected.code, compat.code);
    assert!(
        selected.sections.is_none(),
        "source-map-free supported DOM compiles should come from the S2 selector"
    );
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        None,
        "the production selector must not instantiate the profiling observer"
    );
}

#[test]
fn comments_stay_on_compatibility_without_profiler() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    let source = "<div><!--kept--><span>probe</span></div>";
    let options = DomCompilerOptions {
        comments: true,
        ..Default::default()
    };

    let result = compile(source, options);
    let counters = profiler.counter_summary();

    assert!(
        result.sections.is_some(),
        "comment-preserving compiles must keep compatibility sections"
    );
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        None,
        "compatibility compiles must not instantiate the profiling observer"
    );
}

#[test]
fn sfc_sections_entry_stays_on_compatibility_until_s2_sections_land() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    let result = compile_sfc_sections_entry(
        r#"<button @click="go">{{ label }}</button>"#,
        DomCompilerOptions::default(),
    );
    let counters = profiler.counter_summary();

    assert!(
        result.sections.is_some(),
        "SFC template assembly needs compatibility sections until S2 records them"
    );
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        None,
        "compatibility compiles must not instantiate the profiling observer"
    );
}

struct Compiled {
    preamble: String,
    code: String,
    sections: Option<vize_atelier_core::CodegenSections>,
}

fn compile(source: &str, options: DomCompilerOptions) -> Compiled {
    let allocator = Allocator::new();
    let (_, errors, result) =
        compile_template_with_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options(
            &allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
            None,
            CodegenOptions::default(),
        );
    assert!(errors.is_empty(), "compile errors: {errors:?}");
    Compiled {
        preamble: result.result.preamble.to_string(),
        code: result.result.code.to_string(),
        sections: result.sections,
    }
}

fn compile_sfc_sections_entry(source: &str, options: DomCompilerOptions) -> Compiled {
    let allocator = Allocator::new();
    let (_, errors, result) =
        compile_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options(
            &allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
            None,
            CustomElementMatcher::default(),
            CodegenOptions::default(),
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
