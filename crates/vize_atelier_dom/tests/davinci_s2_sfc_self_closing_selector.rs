//! P2-11 witness: parser-recovered SFC self-closing HTML sections still emit
//! through S2 once the compatibility parser has recorded its diagnostic.

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
fn sfc_sections_entry_uses_s2_for_parser_recovered_html_self_closing_native_tags() {
    let _env_guard = lock_env();
    let _guard = lock_profiler();
    let profiler = global_profiler();

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
        profiler.disable();
        profiler.clear();

        let (compat_error_count, compat) = {
            let _flag = ScopedEnvVar::set(vize_s1_to_s2::DOM_LANE_FLAG, "legacy");
            compile_sfc_sections_entry_with_errors(source, DomCompilerOptions::default())
        };
        assert!(
            compat_error_count > 0,
            "{label} must surface the compatibility parser diagnostic on legacy"
        );

        profiler.enable();
        let (error_count, selected) =
            compile_sfc_sections_entry_with_errors(source, DomCompilerOptions::default());
        let counters = profiler.counter_summary();
        profiler.disable();
        profiler.clear();

        assert!(
            error_count > 0,
            "{label} must surface the compatibility parser diagnostic"
        );
        assert!(
            selected.sections.is_some(),
            "{label} must keep compatibility sections"
        );
        assert_eq!(selected.preamble, compat.preamble, "{label} preamble");
        assert_eq!(selected.code, compat.code, "{label} code");
        assert_eq!(selected.sections, compat.sections, "{label} sections");
        assert_eq!(
            counter_total(&counters, "davinci.s2_dom.files"),
            Some(1),
            "{label} must use S2 after parser-recorded compatibility diagnostics"
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

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    static ENV_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    ENV_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

struct ScopedEnvVar {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl ScopedEnvVar {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var_os(key);
        // SAFETY: This test holds the local environment lock for the full
        // lifetime of the scoped override.
        unsafe { std::env::set_var(key, value) };
        Self { key, previous }
    }
}

impl Drop for ScopedEnvVar {
    fn drop(&mut self) {
        // SAFETY: The guard is dropped before the local environment lock,
        // restoring the process environment while mutations are serialized.
        unsafe {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }
}
