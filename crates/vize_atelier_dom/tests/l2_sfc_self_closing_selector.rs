//! P2-11 witness: parser-recovered SFC self-closing HTML sections still emit
//! through L2 once the compatibility parser has recorded its diagnostic.

#![expect(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "insta and fixtures use format!; fixtures use std strings"
)]

use std::sync::Arc;

use vize_atelier_core::options::{CodegenOptions, CustomElementMatcher, TemplateSyntaxMode};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_sfc_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
    compile_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_and_codegen_options,
};
use vize_l0::Allocator;
use vize_l0::profiler::{CounterSummary, Profiler, ProfilerScope};

#[test]
fn sfc_sections_entry_uses_l2_for_parser_recovered_html_self_closing_native_tags() {
    let profiler = Arc::new(Profiler::new());
    let _scope = ProfilerScope::install(&profiler);

    for (label, source) in [
        ("html_root", r#"<div />"#),
        (
            "svg_html_reentry",
            r#"<svg><foreignObject><div /></foreignObject></svg>"#,
        ),
        (
            "mathml_html_reentry",
            r#"<math><annotation-xml><div /></annotation-xml></math>"#,
        ),
    ] {
        profiler.set_enabled(false);
        profiler.clear();

        let (compat_error_count, compat) =
            compile_sfc_compat_entry_with_errors(source, DomCompilerOptions::default());
        assert!(
            compat_error_count > 0,
            "{label} must surface the compatibility parser diagnostic"
        );

        profiler.set_enabled(true);
        let (error_count, selected) =
            compile_sfc_sections_entry_with_errors(source, DomCompilerOptions::default());
        let counters = profiler.counter_summary();
        profiler.set_enabled(false);
        profiler.clear();

        assert!(
            error_count > 0,
            "{label} must surface the compatibility parser diagnostic"
        );
        assert!(
            selected.sections.is_some(),
            "{label} must keep SFC section boundaries"
        );
        assert_eq!(selected.preamble, compat.preamble, "{label} preamble");
        assert_eq!(selected.code, compat.code, "{label} code");
        assert_eq!(
            counter_total(&counters, "l2.dom.files"),
            Some(1),
            "{label} must use L2 after parser-recorded compatibility diagnostics"
        );
    }
}

struct Compiled {
    preamble: String,
    code: String,
    sections: Option<vize_atelier_core::CodegenSections>,
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

fn compile_sfc_compat_entry_with_errors(
    source: &str,
    options: DomCompilerOptions,
) -> (usize, Compiled) {
    let allocator = Allocator::new();
    let (_, errors, result) =
        compile_template_with_custom_elements_and_template_syntax_and_hoisted_scope_id_and_codegen_options(
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
            preamble: result.preamble.to_string(),
            code: result.code.to_string(),
            sections: None,
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
