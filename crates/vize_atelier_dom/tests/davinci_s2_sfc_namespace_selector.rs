//! P2-11 witness: SFC section selection keeps HTML and foreign namespace
//! self-closing guards distinct while admitting SVG and MathML fast paths.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use vize_atelier_core::options::{CodegenOptions, CustomElementMatcher, TemplateSyntaxMode};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_sfc_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
};
use vize_s0::Allocator;
use vize_s0::profiler::{CounterSummary, global_profiler};

#[test]
fn sfc_sections_entry_keeps_custom_foreign_descendants_on_s2() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    for (label, source) in [
        (
            "svg_custom_descendant",
            r#"<svg><g><my-node><div /></my-node></g></svg>"#,
        ),
        (
            "mathml_custom_descendant",
            r#"<math><mrow><my-node><div /></my-node></mrow></math>"#,
        ),
    ] {
        let compat = compile_sfc_sections_entry(
            source,
            DomCompilerOptions {
                source_map: true,
                ..Default::default()
            },
        );

        profiler.enable();
        let selected = compile_sfc_sections_entry(source, DomCompilerOptions::default());
        let counters = profiler.counter_summary();
        profiler.disable();
        profiler.clear();

        assert_eq!(selected.preamble, compat.preamble, "{label} preamble");
        assert_eq!(selected.code, compat.code, "{label} code");
        assert_eq!(selected.sections, compat.sections, "{label} sections");
        assert_eq!(
            counter_total(&counters, "davinci.s2_dom.files"),
            Some(1),
            "{label} must keep the S2 SFC sections fast path"
        );
    }
}

#[test]
fn sfc_sections_entry_ignores_lt_inside_foreign_attributes() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    for (label, source) in [
        (
            "svg_attribute",
            r#"<svg data-probe="<é"><circle :r="radius" /></svg>"#,
        ),
        (
            "mathml_attribute",
            r#"<math data-probe="<é"><msub :data-depth="depth"><mi>x</mi></msub></math>"#,
        ),
    ] {
        let compat = compile_sfc_sections_entry(
            source,
            DomCompilerOptions {
                source_map: true,
                ..Default::default()
            },
        );

        profiler.enable();
        let selected = compile_sfc_sections_entry(source, DomCompilerOptions::default());
        let counters = profiler.counter_summary();
        profiler.disable();
        profiler.clear();

        assert_eq!(selected.preamble, compat.preamble, "{label} preamble");
        assert_eq!(selected.code, compat.code, "{label} code");
        assert_eq!(selected.sections, compat.sections, "{label} sections");
        assert_eq!(
            counter_total(&counters, "davinci.s2_dom.files"),
            Some(1),
            "{label} must use the S2 SFC sections fast path"
        );
    }
}

#[test]
fn sfc_sections_entry_ignores_native_raw_text_bodies() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    for (label, source) in [
        (
            "textarea_literal_tag",
            r#"<textarea><div /></textarea><p>{{ label }}</p>"#,
        ),
        (
            "title_literal_tag",
            r#"<title><div /></title><p>{{ label }}</p>"#,
        ),
    ] {
        let compat = compile_sfc_sections_entry(
            source,
            DomCompilerOptions {
                source_map: true,
                ..Default::default()
            },
        );

        profiler.enable();
        let selected = compile_sfc_sections_entry(source, DomCompilerOptions::default());
        let counters = profiler.counter_summary();
        profiler.disable();
        profiler.clear();

        assert_eq!(selected.preamble, compat.preamble, "{label} preamble");
        assert_eq!(selected.code, compat.code, "{label} code");
        assert_eq!(selected.sections, compat.sections, "{label} sections");
        assert_eq!(
            counter_total(&counters, "davinci.s2_dom.files"),
            Some(1),
            "{label} must keep the S2 SFC sections fast path"
        );
    }
}

#[test]
fn sfc_sections_entry_pops_html_reentry_closes_case_insensitively() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    let source = r#"<svg><foreignObject><div>label</DIV></foreignObject><path :d="path" /></svg>"#;
    let compat = compile_sfc_sections_entry(
        source,
        DomCompilerOptions {
            source_map: true,
            ..Default::default()
        },
    );

    profiler.enable();
    let selected = compile_sfc_sections_entry(source, DomCompilerOptions::default());
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();

    assert_eq!(selected.preamble, compat.preamble);
    assert_eq!(selected.code, compat.code);
    assert_eq!(selected.sections, compat.sections);
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        Some(1),
        "case-insensitive HTML closes inside foreign re-entry must not poison the S2 selector"
    );
}

#[test]
fn sfc_sections_entry_uses_parser_backed_s2_for_root_html_title() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();

    let (_, result) =
        compile_sfc_sections_entry_with_errors(r#"<title />"#, DomCompilerOptions::default());
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();

    assert!(
        result.sections.is_some(),
        "root HTML title must keep compatibility sections"
    );
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        Some(1),
        "root HTML title must use S2 after parser recovery, not the direct foreign fast path"
    );
}

struct Compiled {
    preamble: String,
    code: String,
    sections: Option<vize_atelier_core::CodegenSections>,
}

fn compile_sfc_sections_entry(source: &str, options: DomCompilerOptions) -> Compiled {
    let (error_count, compiled) = compile_sfc_sections_entry_with_errors(source, options);
    assert_eq!(error_count, 0, "compile errors");
    compiled
}

fn compile_sfc_sections_entry_with_errors(
    source: &str,
    options: DomCompilerOptions,
) -> (usize, Compiled) {
    let allocator = Allocator::new();
    let (errors, result) =
        compile_sfc_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options(
            &allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
            None,
            CustomElementMatcher::default(),
            CodegenOptions::default(),
        );
    (
        errors.len(),
        Compiled {
            preamble: result.result.preamble.to_string(),
            code: result.result.code.to_string(),
            sections: result.sections,
        },
    )
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
