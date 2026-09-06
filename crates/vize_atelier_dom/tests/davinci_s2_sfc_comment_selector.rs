//! P2-11 witness: SFC section selection keeps comment-preserving DOM compiles
//! on the compatibility path until S2 preserves authored comments.

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
fn comment_preserving_sfc_sections_stay_on_compatibility_when_profiled() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    let source = "<div><!--kept--><span>probe</span></div>";
    let compat = compile_sfc_sections_entry(
        source,
        DomCompilerOptions {
            comments: true,
            source_map: true,
            ..Default::default()
        },
    );

    profiler.enable();
    let selected = compile_sfc_sections_entry(
        source,
        DomCompilerOptions {
            comments: true,
            ..Default::default()
        },
    );
    let counters = profiler.counter_summary();
    profiler.disable();
    profiler.clear();

    assert_eq!(selected.preamble, compat.preamble);
    assert_eq!(selected.code, compat.code);
    assert_eq!(selected.sections, compat.sections);
    assert_eq!(
        selected.code,
        r#"function render(_ctx, _cache, $props, $setup, $data, $options) {
  return (_openBlock(), _createElementBlock("div", null, [
    _createCommentVNode("kept"),
    _createElementVNode("span", null, "probe")
  ]))
}"#,
        "comment-preserving SFC compiles must keep the exact comment vnode output"
    );
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        None,
        "comment-preserving SFC sections must stay on compatibility until S2 preserves comments"
    );
}

struct Compiled {
    preamble: String,
    code: String,
    sections: Option<vize_atelier_core::CodegenSections>,
}

fn compile_sfc_sections_entry(source: &str, options: DomCompilerOptions) -> Compiled {
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
