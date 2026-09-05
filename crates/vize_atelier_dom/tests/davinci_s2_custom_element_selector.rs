//! P2-11 witness: declarative custom-element matchers remain an explicit
//! compatibility boundary for the S2 DOM production selector.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use vize_atelier_core::codegen::CodegenResultWithSections;
use vize_atelier_core::options::{CodegenOptions, CustomElementMatcher, TemplateSyntaxMode};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_sfc_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
    compile_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
};
use vize_s0::Allocator;
use vize_s0::profiler::{CounterSummary, global_profiler};

static PROFILER_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn custom_element_matcher_keeps_template_sections_on_compatibility_codegen() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    let source = r#"<ion-button @click="go">{{ label }}</ion-button>"#;
    let compat = compile_template_sections(
        source,
        DomCompilerOptions {
            source_map: true,
            ..Default::default()
        },
        custom_elements(),
    );

    profiler.enable();
    let selected =
        compile_template_sections(source, DomCompilerOptions::default(), custom_elements());
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();

    assert_eq!(selected.preamble, compat.preamble);
    assert_eq!(selected.code, compat.code);
    assert_eq!(selected.sections, compat.sections);
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        None,
        "custom-element matchers are not covered by the S2 production selector yet"
    );
}

#[test]
fn custom_element_matcher_keeps_sfc_sections_on_compatibility_codegen() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    let source = r#"<ion-button @click="go">{{ label }}</ion-button>"#;
    let compat = compile_sfc_sections(
        source,
        DomCompilerOptions {
            source_map: true,
            ..Default::default()
        },
        custom_elements(),
    );

    profiler.enable();
    let selected = compile_sfc_sections(source, DomCompilerOptions::default(), custom_elements());
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();

    assert_eq!(selected.preamble, compat.preamble);
    assert_eq!(selected.code, compat.code);
    assert_eq!(selected.sections, compat.sections);
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        None,
        "custom-element matchers are not covered by the S2 SFC sections fast path yet"
    );
}

struct Compiled {
    preamble: String,
    code: String,
    sections: Option<vize_atelier_core::CodegenSections>,
}

fn compile_template_sections(
    source: &str,
    options: DomCompilerOptions,
    custom_elements: CustomElementMatcher,
) -> Compiled {
    let allocator = Allocator::new();
    let (_, errors, result) =
        compile_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options(
            &allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
            None,
            custom_elements,
            CodegenOptions::default(),
        );
    assert!(errors.is_empty(), "compile errors: {errors:?}");
    compiled(result)
}

fn compile_sfc_sections(
    source: &str,
    options: DomCompilerOptions,
    custom_elements: CustomElementMatcher,
) -> Compiled {
    let allocator = Allocator::new();
    let (errors, result) =
        compile_sfc_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options(
            &allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
            None,
            custom_elements,
            CodegenOptions::default(),
        );
    assert!(errors.is_empty(), "compile errors: {errors:?}");
    compiled(result)
}

fn compiled(result: CodegenResultWithSections) -> Compiled {
    Compiled {
        preamble: result.result.preamble.to_string(),
        code: result.result.code.to_string(),
        sections: result.sections,
    }
}

fn custom_elements() -> CustomElementMatcher {
    CustomElementMatcher::from_patterns(vec!["ion-*".into()])
}

fn counter_total(counters: &CounterSummary, name: &str) -> Option<u64> {
    counters
        .entries
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.total)
}

fn lock_profiler() -> std::sync::MutexGuard<'static, ()> {
    PROFILER_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
