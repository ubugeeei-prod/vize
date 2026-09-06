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
    compile_sfc_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
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
    assert_eq!(selected.sections, compat.sections);
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        None,
        "the production selector must not instantiate the profiling observer"
    );
}

#[test]
fn source_map_compiles_use_s2_with_the_compatibility_map_when_profiled() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    let source = r#"<button @click="go">{{ label }}</button>"#;
    let source_map_free = compile(source, DomCompilerOptions::default());

    profiler.enable();
    let mapped = compile(
        source,
        DomCompilerOptions {
            source_map: true,
            ..Default::default()
        },
    );
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();

    assert_eq!(mapped.preamble, source_map_free.preamble);
    assert_eq!(mapped.code, source_map_free.code);
    assert!(
        mapped.map.is_some(),
        "source-map compiles must keep producing a map"
    );
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        Some(1),
        "source-map compiles must use S2 once the map is verified against compatibility codegen"
    );
}

#[test]
fn unsupported_options_stay_on_compatibility_without_profiler() {
    let _guard = lock_profiler();
    let profiler = global_profiler();

    let cases = [
        (
            "ssr",
            DomCompilerOptions {
                ssr: true,
                ..Default::default()
            },
        ),
        (
            "patterned_template",
            DomCompilerOptions {
                experimental_patterned_template: true,
                ..Default::default()
            },
        ),
        (
            "custom_renderer",
            DomCompilerOptions {
                custom_renderer: true,
                ..Default::default()
            },
        ),
    ];

    for (label, options) in cases {
        profiler.disable();
        profiler.clear();

        let result = compile(r#"<button @click="go">{{ label }}</button>"#, options);
        let counters = profiler.counter_summary();

        assert!(
            result.sections.is_some(),
            "{label} must stay on the compatibility path until S2 supports it"
        );
        assert_eq!(
            counter_total(&counters, "davinci.s2_dom.files"),
            None,
            "{label} compatibility compiles must not instantiate the profiling observer"
        );
    }
}

#[test]
fn in_tag_comment_compiles_use_s2_codegen() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();
    profiler.enable();

    let selected = compile(
        "<button // keep the parse extension covered\n  @click=\"go\">{{ label }}</button>",
        DomCompilerOptions {
            experimental_in_tag_comments: true,
            ..Default::default()
        },
    );
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();

    assert!(selected.sections.is_some());
    assert_eq!(counter_total(&counters, "davinci.s2_dom.files"), Some(1));
}

#[test]
fn sfc_sections_entry_uses_s2_once_sections_land() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    let source = r#"<button @click="go">{{ label }}</button>"#;
    let compat = compile_sfc_sections_entry(
        source,
        DomCompilerOptions {
            source_map: true,
            ..Default::default()
        },
    );
    let selected = compile_sfc_sections_entry(source, DomCompilerOptions::default());
    let counters = profiler.counter_summary();

    assert_eq!(selected.preamble, compat.preamble);
    assert_eq!(selected.code, compat.code);
    assert_eq!(selected.sections, compat.sections);
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        None,
        "the SFC sections entry must not instantiate the profiling observer"
    );
}

#[test]
fn sfc_sections_entry_uses_s2_for_foreign_namespace_templates() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    for (label, source) in [
        ("svg", r#"<svg><circle :r="radius" /></svg>"#),
        (
            "mathml",
            r#"<math><msub :data-depth="depth"><mi>x</mi></msub></math>"#,
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

struct Compiled {
    preamble: String,
    code: String,
    map: Option<String>,
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
        map: result.result.map.map(|map| map.to_string()),
        sections: result.sections,
    }
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
            map: result.result.map.map(|map| map.to_string()),
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
