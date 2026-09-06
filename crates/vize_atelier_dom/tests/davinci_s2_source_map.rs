//! P2-11 witness: source-map requests can use S2 DOM output while retaining
//! the shipped source-map contract.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_core::{
    CodegenSections,
    codegen::CodegenResultWithSections,
    options::{CodegenOptions, CustomElementMatcher, TemplateSyntaxMode},
};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_sfc_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
    compile_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
    compile_template_with_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
};
use vize_s0::{
    Allocator,
    profiler::{CounterSummary, global_profiler},
};

#[test]
fn source_map_template_compile_uses_s2_with_the_compatibility_map() {
    let source = r#"<section id="app">{{ msg }}<span :title="title">ok</span></section>"#;
    let codegen = CodegenOptions {
        filename: "MappedTemplate.vue".into(),
        ..Default::default()
    };
    let compat = compile_template_compat(
        source,
        DomCompilerOptions {
            source_map: true,
            ..Default::default()
        },
        codegen.clone(),
    );
    let (selected, counters) = compile_template_selected_with_profile(
        source,
        DomCompilerOptions {
            source_map: true,
            ..Default::default()
        },
        codegen,
    );

    assert_selected_source_map_matches_compat(selected, compat, counters);
}

#[test]
fn source_map_sfc_sections_compile_uses_s2_with_the_compatibility_map() {
    let source = r#"<section id="app">{{ msg }}<span :title="title">ok</span></section>"#;
    let codegen = CodegenOptions {
        filename: "MappedSfcTemplate.vue".into(),
        ..Default::default()
    };
    let compat = compile_sfc_compat(
        source,
        DomCompilerOptions {
            source_map: true,
            ..Default::default()
        },
        codegen.clone(),
    );
    let (selected, counters) = compile_sfc_selected_with_profile(
        source,
        DomCompilerOptions {
            source_map: true,
            ..Default::default()
        },
        codegen,
    );

    assert_selected_source_map_matches_compat(selected, compat, counters);
}

#[test]
fn in_tag_comment_source_map_compile_uses_s2_with_the_compatibility_map() {
    let source =
        "<button // keep the parse extension covered\n  @click=\"go\">{{ label }}</button>";
    let options = DomCompilerOptions {
        source_map: true,
        experimental_in_tag_comments: true,
        ..Default::default()
    };
    let codegen = CodegenOptions {
        filename: "InTagComment.vue".into(),
        ..Default::default()
    };
    let compat = compile_template_compat(source, options.clone(), codegen.clone());
    let (selected, counters) = compile_template_selected_with_profile(source, options, codegen);

    assert_selected_source_map_matches_compat(selected, compat, counters);
}

fn assert_selected_source_map_matches_compat(
    selected: Compiled,
    compat: Compiled,
    counters: CounterSummary,
) {
    assert_eq!(selected.preamble, compat.preamble);
    assert_eq!(selected.code, compat.code);
    assert_eq!(selected.sections, compat.sections);
    assert_eq!(selected.map, compat.map);

    assert!(
        selected.map.is_some(),
        "source-map requests must remain additive"
    );
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        Some(1),
        "source-map requests must enter the S2 DOM production selector"
    );

    let map = selected.map.expect("source map");
    let parsed: serde_json::Value = serde_json::from_str(&map).expect("map must be valid JSON");
    assert_eq!(parsed["version"], 3);
    assert!(
        parsed["mappings"].as_str().is_some_and(|m| !m.is_empty()),
        "the selected source map must keep non-empty mappings"
    );
}

struct Compiled {
    preamble: String,
    code: String,
    map: Option<String>,
    sections: Option<CodegenSections>,
}

fn compile_template_selected_with_profile(
    source: &str,
    options: DomCompilerOptions,
    codegen: CodegenOptions,
) -> (Compiled, CounterSummary) {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();
    profiler.enable();
    let selected = compile_template_selected(source, options, codegen);
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();
    (selected, counters)
}

fn compile_sfc_selected_with_profile(
    source: &str,
    options: DomCompilerOptions,
    codegen: CodegenOptions,
) -> (Compiled, CounterSummary) {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();
    profiler.enable();
    let selected = compile_sfc_selected(source, options, codegen);
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();
    (selected, counters)
}

fn compile_template_selected(
    source: &str,
    options: DomCompilerOptions,
    codegen: CodegenOptions,
) -> Compiled {
    let allocator = Allocator::new();
    let (_, errors, result) =
        compile_template_with_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options(
            &allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
            None,
            codegen,
        );
    assert!(errors.is_empty(), "compile errors: {errors:?}");
    compiled(result)
}

fn compile_template_compat(
    source: &str,
    options: DomCompilerOptions,
    codegen: CodegenOptions,
) -> Compiled {
    let allocator = Allocator::new();
    let (_, errors, result) =
        compile_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options(
            &allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
            None,
            non_matching_custom_elements(),
            codegen,
        );
    assert!(errors.is_empty(), "compile errors: {errors:?}");
    compiled(result)
}

fn compile_sfc_selected(
    source: &str,
    options: DomCompilerOptions,
    codegen: CodegenOptions,
) -> Compiled {
    let allocator = Allocator::new();
    let (errors, result) =
        compile_sfc_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options(
            &allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
            None,
            CustomElementMatcher::default(),
            codegen,
        );
    assert!(errors.is_empty(), "compile errors: {errors:?}");
    compiled(result)
}

fn compile_sfc_compat(
    source: &str,
    options: DomCompilerOptions,
    codegen: CodegenOptions,
) -> Compiled {
    let allocator = Allocator::new();
    let (errors, result) =
        compile_sfc_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options(
            &allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
            None,
            non_matching_custom_elements(),
            codegen,
        );
    assert!(errors.is_empty(), "compile errors: {errors:?}");
    compiled(result)
}

fn compiled(result: CodegenResultWithSections) -> Compiled {
    Compiled {
        preamble: result.result.preamble.to_string(),
        code: result.result.code.to_string(),
        map: result.result.map.map(|map| map.to_string()),
        sections: result.sections,
    }
}

fn non_matching_custom_elements() -> CustomElementMatcher {
    CustomElementMatcher::from_patterns(vec!["x-never-*".into()])
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
