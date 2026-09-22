//! The `analyzeSfc` `spolveroProfile` member (C-3): the ladder run behind the
//! `spolvero` feed, timed by the host clock the entry passes, exported as a
//! P0-11 profile document. A deterministic clock pins it exactly; schema
//! validation of the same exporter output is `vize_curator`'s
//! `spolvero_timing` suite.

use std::cell::Cell;

use super::analyze::analyze_sfc_json_with_clock;

const SOURCE: &str = "<template>\n  <div>{{ msg }}</div>\n</template>\n";

fn squares() -> impl Fn() -> u64 {
    let reads = Cell::new(0_u64);
    move || {
        let k = reads.get();
        reads.set(k + 1);
        k * k * 1000
    }
}

fn span(stage: &str, pass: &str, nanos: u64, bucket: u64) -> serde_json::Value {
    serde_json::json!({
        "key": "davinci.spolvero.step",
        "count": 1,
        "wall_ns": {
            "total": nanos, "self": nanos, "min": nanos, "max": nanos,
            "p50": bucket, "p95": bucket, "p99": bucket,
        },
        "alloc": null,
        "attribution": { "stage": stage, "pass": pass, "block": "template" },
    })
}

fn profile(spans: Vec<serde_json::Value>) -> serde_json::Value {
    serde_json::json!({
        "schema_version": 1,
        "tool": "vize",
        "tool_version": env!("CARGO_PKG_VERSION"),
        "command": "analyze-sfc",
        "budget": { "max_spans": 512, "max_counters": 256 },
        "truncation": { "dropped_spans": 0, "dropped_counters": 0 },
        "spans": spans,
        "counters": [],
        "allocation": null,
    })
}

#[test]
fn the_analyze_result_times_every_ladder_step_by_the_host_clock() {
    let clock = squares();
    let result =
        analyze_sfc_json_with_clock(SOURCE, "src/App.vue", false, false, &clock).expect("analysis");
    // Reads 0..8 bracket parse, S2 lower, the one selected pass, S3 lower.
    assert_eq!(
        result["spolveroProfile"],
        profile(vec![
            span("s3", "lower", 13_000, 16_000),
            span("s2", "hoist-static", 9_000, 16_000),
            span("s2", "lower", 5_000, 8_000),
            span("s1", "parse", 1_000, 1_000),
        ])
    );
}

#[test]
fn a_template_less_sfc_profiles_no_step() {
    let clock = squares();
    let result = analyze_sfc_json_with_clock(
        "<script setup>\nconst n = 1\n</script>\n",
        "src/Logic.vue",
        false,
        false,
        &clock,
    )
    .expect("analysis");
    assert_eq!(result["spolveroProfile"], profile(vec![]));
}
