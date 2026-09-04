//! P2-12b witness: profile counters describe the S2 emitter used by the
//! profiled source-map-free DOM output.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use davinci_harness::fixtures::{LADDER, template_block};
use vize_atelier_dom::{
    DomCompilerOptions, compile_template, compile_template_with_options,
    compile_template_with_options_and_hoisted_scope_id,
};
use vize_s0::Allocator;
use vize_s0::profiler::{CounterSummary, global_profiler};

static PROFILER_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

const S2_DOM_EMIT_COUNTS: [(&str, u64, u64); 6] = [
    ("small", 1, 5),
    ("medium", 1, 33),
    ("large", 1, 54),
    ("stress-deep", 1, 72),
    ("stress-wide", 1, 2),
    ("stress-interp", 1, 201),
];

#[test]
fn profile_reports_real_s2_dom_walks() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    profiler.clear();

    let unobserved_allocator = Allocator::new();
    let (_, unobserved_errors, unobserved) =
        compile_template(&unobserved_allocator, "<div>{{ msg }}</div>");
    let unobserved_counters = profiler.counter_summary();
    assert!(unobserved_errors.is_empty());
    assert_eq!(
        counter_total(&unobserved_counters, "davinci.s2_dom.files"),
        None,
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
fn profile_reports_ladder_s2_dom_walk_budget() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();

    for fixture in &LADDER {
        let template =
            template_block(fixture.source).expect("every ladder fixture has a template block");
        let expected = s2_dom_emit_count(fixture.name);
        let compat_allocator = Allocator::new();
        let (_, compat_errors, compat) = compile_template(&compat_allocator, template);
        assert!(
            compat_errors.is_empty(),
            "{} compatibility compile should be diagnostic-free",
            fixture.name
        );

        profiler.clear();
        profiler.enable();

        let allocator = Allocator::new();
        let (_, errors, result) = compile_template(&allocator, template);

        profiler.disable();
        let counters = profiler.counter_summary();

        assert!(
            errors.is_empty(),
            "{} profiled compile should be diagnostic-free",
            fixture.name
        );
        assert_eq!(
            result.preamble, compat.preamble,
            "{} profiled S2 emit must keep the compatibility preamble",
            fixture.name
        );
        assert_eq!(
            result.code, compat.code,
            "{} profiled S2 emit must keep the compatibility code",
            fixture.name
        );
        assert_eq!(counter(&counters, "davinci.s2_dom.files"), 1);
        assert_eq!(counter(&counters, "davinci.s2_dom.transform.walks"), 6);
        assert_eq!(counter(&counters, "davinci.s2_dom.transform.passes"), 6);
        assert_eq!(counter(&counters, "davinci.s2_dom.emit.walks"), expected.0);
        assert_eq!(counter(&counters, "davinci.s2_dom.emit.visits"), expected.1);
        assert_eq!(
            counter(&counters, "davinci.s2_dom.total.walks"),
            6 + expected.0
        );
    }

    profiler.clear();
}

#[test]
fn source_map_disabled_scope_id_uses_compatibility_codegen() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    let source = r#"<div class="scoped">{{ msg }}</div>"#;
    let scoped_options = DomCompilerOptions {
        scope_id: Some("data-v-direct".into()),
        ..Default::default()
    };
    let compat_allocator = Allocator::new();
    let (_, compat_errors, compat) = compile_template_with_options(
        &compat_allocator,
        source,
        DomCompilerOptions {
            source_map: true,
            ..scoped_options.clone()
        },
    );
    assert!(compat_errors.is_empty());

    profiler.clear();
    profiler.enable();

    let allocator = Allocator::new();
    let (_, errors, result) = compile_template_with_options(&allocator, source, scoped_options);

    profiler.disable();
    let counters = profiler.counter_summary();

    assert!(errors.is_empty());
    assert_eq!(result.preamble, compat.preamble);
    assert_eq!(result.code, compat.code);
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        None,
        "direct scope_id compiles must not be routed through S2 yet"
    );
}

#[test]
fn source_map_disabled_hoisted_scope_id_stays_on_s2_codegen() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    let source = r#"<div :class="{ active }"><svg><rect class="marker" x="1" /></svg></div>"#;
    let compat_allocator = Allocator::new();
    let (_, compat_errors, compat) = compile_template_with_options_and_hoisted_scope_id(
        &compat_allocator,
        source,
        DomCompilerOptions {
            source_map: true,
            ..Default::default()
        },
        Some("data-v-hoist".into()),
    );
    assert!(compat_errors.is_empty());

    profiler.clear();
    profiler.enable();

    let allocator = Allocator::new();
    let (_, errors, result) = compile_template_with_options_and_hoisted_scope_id(
        &allocator,
        source,
        DomCompilerOptions::default(),
        Some("data-v-hoist".into()),
    );

    profiler.disable();
    let counters = profiler.counter_summary();

    assert!(errors.is_empty());
    assert_eq!(counter(&counters, "davinci.s2_dom.files"), 1);
    assert_eq!(result.preamble, compat.preamble);
    assert_eq!(result.code, compat.code);
}

#[test]
fn source_map_disabled_comments_use_compatibility_codegen() {
    let _guard = lock_profiler();
    let profiler = global_profiler();
    profiler.disable();
    let source = "<div><!--kept--><span>probe</span></div>";
    let options = DomCompilerOptions {
        comments: true,
        ..Default::default()
    };
    let compat_allocator = Allocator::new();
    let (_, compat_errors, compat) = compile_template_with_options(
        &compat_allocator,
        source,
        DomCompilerOptions {
            source_map: true,
            ..options.clone()
        },
    );
    assert!(compat_errors.is_empty());

    profiler.clear();
    profiler.enable();

    let allocator = Allocator::new();
    let (_, errors, result) = compile_template_with_options(&allocator, source, options);

    profiler.disable();
    let counters = profiler.counter_summary();

    assert!(errors.is_empty());
    assert_eq!(result.preamble, compat.preamble);
    assert_eq!(result.code, compat.code);
    assert_eq!(
        counter_total(&counters, "davinci.s2_dom.files"),
        None,
        "comment-preserving compiles must stay on compatibility codegen"
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

fn counter_total(counters: &CounterSummary, name: &str) -> Option<u64> {
    counters
        .entries
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.total)
}

fn s2_dom_emit_count(fixture: &str) -> (u64, u64) {
    S2_DOM_EMIT_COUNTS
        .iter()
        .find(|(name, _, _)| *name == fixture)
        .map(|(_, walks, visits)| (*walks, *visits))
        .unwrap_or_else(|| panic!("{fixture} has no pinned S2 DOM emit count"))
}

fn lock_profiler() -> std::sync::MutexGuard<'static, ()> {
    PROFILER_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
