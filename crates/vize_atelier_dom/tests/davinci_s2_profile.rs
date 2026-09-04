//! P2-12b witness: profile counters describe the S2 emitter that produced the
//! normal DOM output, rather than a separately planned legacy pipeline.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use vize_atelier_dom::{
    DomCompilerOptions, compile_template, compile_template_with_options,
    compile_template_with_options_and_hoisted_scope_id,
};
use vize_s0::Allocator;
use vize_s0::profiler::{CounterSummary, global_profiler};

static PROFILER_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn profile_reports_real_s2_dom_walks() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    let unobserved_allocator = Allocator::new();
    let (_, unobserved_errors, unobserved) =
        compile_template(&unobserved_allocator, "<div>{{ msg }}</div>");
    assert!(unobserved_errors.is_empty());
    assert!(
        profiler
            .counter_summary()
            .entries
            .iter()
            .all(|entry| !entry.name.starts_with("davinci.s2_dom.")),
        "normal DOM compilation must not instantiate the profiling observer"
    );

    profiler.clear();
    profiler.enable();

    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(&allocator, "<div>{{ msg }}</div>");

    profiler.disable();
    let counters = profiler.counter_summary();

    assert!(errors.is_empty());
    assert!(!result.code.is_empty());
    assert_eq!(result.code, unobserved.code);
    assert_eq!(result.preamble, unobserved.preamble);
    assert_eq!(counter(&counters, "davinci.s2_dom.files"), 1);
    assert_eq!(counter(&counters, "davinci.s2_dom.transform.walks"), 6);
    assert_eq!(counter(&counters, "davinci.s2_dom.transform.passes"), 6);
    assert_eq!(counter(&counters, "davinci.s2_dom.emit.walks"), 1);
    assert!(counter(&counters, "davinci.s2_dom.emit.visits") > 0);
    assert_eq!(counter(&counters, "davinci.s2_dom.total.walks"), 7);
}

#[test]
fn source_map_disabled_scope_id_uses_compatibility_codegen() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();
    profiler.enable();

    let allocator = Allocator::new();
    let (_, errors, result) = compile_template_with_options(
        &allocator,
        r#"<div class="scoped">{{ msg }}</div>"#,
        DomCompilerOptions {
            scope_id: Some("data-v-direct".into()),
            ..Default::default()
        },
    );

    profiler.disable();
    let counters = profiler.counter_summary();

    assert!(errors.is_empty());
    let program = format!("{}\n{}", result.preamble, result.code);
    assert!(
        program.contains("\"data-v-direct\": \"\""),
        "direct scope_id must stay on compatibility codegen until S2 owns runtime scoped attrs:\n{}",
        program
    );
    assert!(
        !has_counter(&counters, "davinci.s2_dom.files"),
        "direct scope_id compiles must not be routed through S2 yet"
    );
}

#[test]
fn source_map_disabled_hoisted_scope_id_stays_on_s2_codegen() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();
    profiler.enable();

    let allocator = Allocator::new();
    let (_, errors, result) = compile_template_with_options_and_hoisted_scope_id(
        &allocator,
        r#"<div :class="{ active }"><svg><rect class="marker" x="1" /></svg></div>"#,
        DomCompilerOptions::default(),
        Some("data-v-hoist".into()),
    );

    profiler.disable();
    let counters = profiler.counter_summary();

    assert!(errors.is_empty());
    assert_eq!(counter(&counters, "davinci.s2_dom.files"), 1);
    assert!(
        result.preamble.contains("\"data-v-hoist\""),
        "hoisted_scope_id must be baked into static VNode hoists emitted by S2:\n{}",
        result.preamble
    );
}

fn counter(counters: &CounterSummary, name: &str) -> u64 {
    counters
        .entries
        .iter()
        .find(|entry| entry.name == name)
        .unwrap_or_else(|| panic!("missing {name} profile counter"))
        .total
}

fn has_counter(counters: &CounterSummary, name: &str) -> bool {
    counters.entries.iter().any(|entry| entry.name == name)
}

fn lock_profiler() -> std::sync::MutexGuard<'static, ()> {
    PROFILER_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
