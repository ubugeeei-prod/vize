//! P2-11 witness: source-map requests can use L2 DOM output while retaining
//! the shipped source-map contract.

#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests assert by panicking; fixtures use std strings"
)]

use vize_atelier_core::{
    CodegenSections,
    codegen::CodegenResultWithSections,
    options::{
        CodegenExperimentalOptions, CodegenMode, CodegenOptions, CustomElementMatcher,
        TemplateSyntaxMode,
    },
};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_sfc_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
    compile_sfc_template_with_preflight_and_environment,
    compile_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
    compile_template_with_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
};
use vize_l0::{
    Allocator,
    profiler::{CounterSummary, global_profiler},
};

#[test]
fn source_map_template_compile_uses_l2_with_the_compatibility_map() {
    let _guard = lock_profiler();
    reset_profiler();
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
fn source_map_sfc_sections_compile_uses_l2_with_the_compatibility_map() {
    let _guard = lock_profiler();
    reset_profiler();
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
fn in_tag_comment_source_map_compile_uses_l2_with_the_compatibility_map() {
    let _guard = lock_profiler();
    reset_profiler();
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

#[test]
fn production_source_map_preserves_conditional_comment_bytes() {
    let _guard = lock_profiler();
    reset_profiler();
    let source = r#"<div><p v-if="open">open</p><span>always</span></div>"#;
    let compile = |source_map| {
        compile_sfc_selected_in_environment(
            source,
            DomCompilerOptions {
                mode: CodegenMode::Module,
                prefix_identifiers: true,
                source_map,
                ..Default::default()
            },
            CodegenOptions {
                filename: "ProductionConditional.vue".into(),
                ..Default::default()
            },
            false,
        )
    };
    let plain = compile(false);
    let profile = ProfileScope::enable();
    let mapped = compile(true);
    let counters = profile.counter_summary();
    let expected = r#"import { createElementVNode as _createElementVNode, openBlock as _openBlock, createElementBlock as _createElementBlock, createCommentVNode as _createCommentVNode } from "vue"

export function render(_ctx, _cache) {
  return (_openBlock(), _createElementBlock("div", null, [
    (_ctx.open)
      ? (_openBlock(), _createElementBlock("p", { key: 0 }, "open"))
      : _createCommentVNode("", true),
    _createElementVNode("span", null, "always")
  ]))
}"#;
    let whole_module = |compiled: &Compiled| {
        let mut module = compiled.preamble.clone();
        module.push('\n');
        module.push_str(&compiled.code);
        module
    };

    assert!(mapped.map.is_some(), "the requested map must be attached");
    assert_eq!(whole_module(&plain), expected);
    assert_eq!(whole_module(&mapped), expected, "maps must remain additive");
    assert_eq!(plain.sections, mapped.sections);
    assert_eq!(
        (
            counter_total(&counters, "l2.dom.accepted"),
            counter_total(&counters, "l2.dom.legacy.emit_refused"),
            counter_total(&counters, "l2.dom.legacy.source_map_mismatch"),
        ),
        (Some(1), None, None),
        "the mapped production compile must retain the accepted L2 module without fallback"
    );
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
        counter_total(&counters, "l2.dom.files"),
        Some(1),
        "source-map requests must enter the L2 DOM production selector"
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
    let _profile = ProfileScope::enable();
    let selected = compile_template_selected(source, options, codegen);
    let counters = _profile.counter_summary();
    (selected, counters)
}

fn compile_sfc_selected_with_profile(
    source: &str,
    options: DomCompilerOptions,
    codegen: CodegenOptions,
) -> (Compiled, CounterSummary) {
    let _profile = ProfileScope::enable();
    let selected = compile_sfc_selected(source, options, codegen);
    let counters = _profile.counter_summary();
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

fn compile_sfc_selected_in_environment(
    source: &str,
    options: DomCompilerOptions,
    codegen: CodegenOptions,
    development: bool,
) -> Compiled {
    let allocator = Allocator::new();
    let (errors, result) = compile_sfc_template_with_preflight_and_environment(
        &allocator,
        source,
        options,
        TemplateSyntaxMode::Standard,
        None,
        CustomElementMatcher::default(),
        codegen,
        CodegenExperimentalOptions::default(),
        false,
        None,
        None,
        development,
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
    CustomElementMatcher::from_static_predicate(is_never_custom_element)
}

fn is_never_custom_element(_tag: &str) -> bool {
    false
}

fn counter_total(counters: &CounterSummary, name: &str) -> Option<u64> {
    counters
        .entries
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.total)
}

fn reset_profiler() {
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();
}

struct ProfileScope;

impl ProfileScope {
    fn enable() -> Self {
        let profiler = global_profiler();
        profiler.clear();
        profiler.enable();
        Self
    }

    fn counter_summary(&self) -> CounterSummary {
        global_profiler().counter_summary()
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        reset_profiler();
    }
}

fn lock_profiler() -> std::sync::MutexGuard<'static, ()> {
    static PROFILER_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    PROFILER_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
