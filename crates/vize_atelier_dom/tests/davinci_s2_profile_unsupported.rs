//! P2-11 witness: profiling must not make unsupported DOM option shapes look
//! like S2 production-selector coverage.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use vize_atelier_dom::{DomCompilerOptions, compile_template_with_options};
use vize_s0::Allocator;
use vize_s0::profiler::{CounterSummary, global_profiler};

static PROFILER_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn profiled_unsupported_options_stay_on_compatibility_codegen() {
    let _guard = lock_profiler();
    for (label, options) in [
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
            "in_tag_comments",
            DomCompilerOptions {
                experimental_in_tag_comments: true,
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
    ] {
        let profile = ProfileScope::enable();
        let allocator = Allocator::new();
        let source = if label == "in_tag_comments" {
            "<div // keep the parse extension covered\n  id=\"x\">{{ msg }}</div>"
        } else {
            "<div>{{ msg }}</div>"
        };
        let (_, errors, result) = compile_template_with_options(&allocator, source, options);
        let counters = profile.finish();

        assert!(errors.is_empty(), "{label} compile errors: {errors:?}");
        assert!(!result.code.is_empty(), "{label} must still emit code");
        assert_eq!(
            counter_total(&counters, "davinci.s2_dom.files"),
            None,
            "{label} must stay on compatibility even when profiling is enabled"
        );
    }
}

fn counter_total(counters: &CounterSummary, name: &str) -> Option<u64> {
    counters
        .entries
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.total)
}

struct ProfileScope;

impl ProfileScope {
    fn enable() -> Self {
        let profiler = global_profiler();
        profiler.clear();
        profiler.enable();
        Self
    }

    fn finish(self) -> CounterSummary {
        let profiler = global_profiler();
        profiler.disable();
        profiler.counter_summary()
    }
}

impl Drop for ProfileScope {
    fn drop(&mut self) {
        let profiler = global_profiler();
        profiler.disable();
        profiler.clear();
    }
}

fn lock_profiler() -> std::sync::MutexGuard<'static, ()> {
    PROFILER_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
