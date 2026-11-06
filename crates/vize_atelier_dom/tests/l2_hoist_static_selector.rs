//! L2 production selector coverage for disabled static hoists.

#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "insta and fixtures use format!; fixtures use std strings"
)]

use std::sync::Arc;

use vize_atelier_core::options::{CodegenOptions, CustomElementMatcher, TemplateSyntaxMode};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
    compile_template_with_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
};
use vize_l0::Allocator;
use vize_l0::profiler::{CounterSummary, Profiler, ProfilerScope};

#[test]
fn disabled_static_hoists_use_l2_codegen() {
    let profiler = Arc::new(Profiler::new());
    let _scope = ProfilerScope::install(&profiler);

    for (label, source) in [
        (
            "static_tree",
            r#"<section><span class="badge">ok</span><strong>{{ label }}</strong></section>"#,
        ),
        (
            "component_slot",
            r#"<Card><span class="badge">ok</span></Card>"#,
        ),
        (
            "slot_fallback",
            r#"<slot><span class="badge">ok</span></slot>"#,
        ),
        (
            "template_if",
            r#"<template v-if="ok"><span class="badge">ok</span></template>"#,
        ),
        (
            "v_once_nested",
            r#"<Card v-once><span class="badge">ok</span></Card>"#,
        ),
    ] {
        profiler.set_enabled(false);
        profiler.clear();

        let options = DomCompilerOptions {
            hoist_static: false,
            ..Default::default()
        };
        let compat = compile_compat(source, options.clone());

        profiler.set_enabled(true);
        let selected = compile(source, options);
        let counters = profiler.counter_summary();
        profiler.set_enabled(false);
        profiler.clear();

        assert_eq!(selected.preamble, compat.preamble, "{label} preamble");
        assert_eq!(selected.code, compat.code, "{label} code");
        assert_eq!(selected.sections, compat.sections, "{label} sections");
        assert_eq!(
            selected.preamble.matches("_hoisted_").count(),
            0,
            "{label}: disabled static hoists must not synthesize hoist declarations"
        );
        assert_eq!(
            counter_total(&counters, "l2.dom.files"),
            Some(1),
            "{label}: disabled static hoists are a supported L2 production option"
        );
    }
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

fn compile_compat(source: &str, options: DomCompilerOptions) -> Compiled {
    let allocator = Allocator::new();
    let (_, errors, result) =
        compile_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options(
            &allocator,
            source,
            options,
            TemplateSyntaxMode::Standard,
            None,
            CustomElementMatcher::from_static_predicate(is_never_custom_element),
            CodegenOptions::default(),
        );
    assert!(errors.is_empty(), "compat compile errors: {errors:?}");
    Compiled {
        preamble: result.result.preamble.to_string(),
        code: result.result.code.to_string(),
        sections: result.sections,
    }
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
