//! P2-11 witness: legacy dialect compiles stay on compatibility after the DOM
//! lane flag deletion, while stale flag values no longer select the old lane.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::ffi::OsString;
use vize_atelier_core::options::{CodegenOptions, TemplateSyntaxMode};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_template_with_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
};
use vize_s0::Allocator;
use vize_s0::config::VueVersion;
use vize_s0::profiler::{CounterSummary, global_profiler};

#[test]
fn legacy_dialect_stays_on_compatibility_after_flag_deletion() {
    let _env_lock = lock_env();
    let _profiler_lock = lock_profiler();
    let (result, counters) = compile_with_s2_profiler(
        r#"<button @click.native="go">{{ label | cap }}</button>"#,
        DomCompilerOptions {
            dialect: VueVersion::V2,
            ..Default::default()
        },
    );

    assert_compatibility_sections_without_s2_profile(
        &result,
        &counters,
        "legacy dialect compiles must not enter the S2 production selector",
    );
}

#[test]
fn removed_dom_lane_flag_does_not_disarm_vue3_s2_selection() {
    let _env_lock = lock_env();
    let _flag = ScopedEnvVar::set(removed_dom_lane_flag_name(), "legacy");
    let _profiler_lock = lock_profiler();
    let (result, counters) = compile_with_s2_profiler(
        r#"<button @click="go">{{ label }}</button>"#,
        DomCompilerOptions::default(),
    );

    assert!(
        result.sections.is_some(),
        "S2 DOM must keep codegen sections available"
    );
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        Some(1),
        "removed DOM lane env values must not disarm the S2 production selector",
    );
}

fn removed_dom_lane_flag_name() -> &'static str {
    concat!("VIZE", "_DAVINCI", "_DOM")
}

struct Compiled {
    sections: Option<vize_atelier_core::CodegenSections>,
}

fn assert_compatibility_sections_without_s2_profile(
    result: &Compiled,
    counters: &CounterSummary,
    s2_message: &str,
) {
    assert!(
        result.sections.is_some(),
        "the compatibility lane must keep codegen sections available"
    );
    assert_eq!(
        counter_total(counters, "davinci.s2_dom.files"),
        None,
        "{s2_message}"
    );
}

fn compile_with_s2_profiler(
    source: &str,
    options: DomCompilerOptions,
) -> (Compiled, CounterSummary) {
    let profiler = ScopedProfiler::enable();
    let result = compile(source, options);
    let counters = profiler.counter_summary();
    (result, counters)
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

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    static ENV_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    ENV_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

struct ScopedEnvVar {
    key: &'static str,
    previous: Option<OsString>,
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

struct ScopedProfiler;

impl ScopedProfiler {
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

impl Drop for ScopedProfiler {
    fn drop(&mut self) {
        let profiler = global_profiler();
        profiler.disable();
        profiler.clear();
    }
}
