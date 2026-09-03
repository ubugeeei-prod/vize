//! P2-12b witness: profile counters describe the S2 emitter that produced the
//! normal DOM output, rather than a separately planned legacy pipeline.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use vize_atelier_dom::compile_template;
use vize_s0::Allocator;
use vize_s0::profiler::{CounterSummary, global_profiler};

#[test]
fn profile_reports_real_s2_dom_walks() {
    let profiler = global_profiler();
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
    assert_eq!(counter(&counters, "davinci.s2_dom.files"), 1);
    assert_eq!(counter(&counters, "davinci.s2_dom.transform.walks"), 6);
    assert_eq!(counter(&counters, "davinci.s2_dom.transform.passes"), 6);
    assert_eq!(counter(&counters, "davinci.s2_dom.emit.walks"), 1);
    assert!(counter(&counters, "davinci.s2_dom.emit.visits") > 0);
    assert_eq!(counter(&counters, "davinci.s2_dom.total.walks"), 7);
}

fn counter(counters: &CounterSummary, name: &str) -> u64 {
    counters
        .entries
        .iter()
        .find(|entry| entry.name == name)
        .unwrap_or_else(|| panic!("missing {name} profile counter"))
        .total
}
