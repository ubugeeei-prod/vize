//! C-3: the Spolvero ladder's step and walk timings. A deterministic host
//! clock proves each step reads the clock exactly once at each end of its
//! compiler work, in run order, that each walk is the timing observer's
//! group window, and that the timings export as a P0-11 profile document
//! that validates against the committed schema through the TS-15 validator.

use std::cell::Cell;
use std::path::Path;

use davinci_test_support::schema as schema_check;
use vize_curator::inspector::{
    LADDER_STEP_KEY, LADDER_WALK_KEY, LadderStep, ladder_pages, ladder_profile, ladder_run,
};

const TEMPLATE: &str = "<Comp v-model=\"x\"><template #a>hi</template></Comp><p>{{ y }}</p>";

/// Reading `k` returns `k² µs`, so consecutive reads `2k, 2k+1` differ by
/// `(4k+1) µs`: every step's duration names which reads bracketed it.
fn squares() -> impl Fn() -> u64 {
    let reads = Cell::new(0_u64);
    move || {
        let k = reads.get();
        reads.set(k + 1);
        k * k * 1000
    }
}

fn load_schema() -> serde_json::Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../davinci-road/plan/profile-export.schema.json");
    let text = std::fs::read_to_string(path).expect("committed schema reads");
    serde_json::from_str(&text).expect("committed schema is valid JSON")
}

fn step(stage: &'static str, pass: &'static str, nanos: u64) -> LadderStep {
    LadderStep { stage, pass, nanos }
}

#[test]
fn every_step_is_timed_between_its_own_two_clock_reads() {
    let clock = squares();
    let run = ladder_run("src/App.vue", TEMPLATE, &clock);
    assert_eq!(
        run.steps,
        vec![
            step("s1", "parse", 1_000),
            step("s2", "lower", 5_000),
            step("s2", "v-slot", 9_000),
            step("s2", "v-model", 13_000),
            step("s2", "hoist-static", 17_000),
            step("s2", "template-complexity", 21_000),
            step("s3", "lower", 25_000),
        ]
    );
    // Barriers own a walk each. `hoist-static` and `template-complexity`
    // share the third: it opens on read 8 and closes on read 11.
    assert_eq!(
        run.walks,
        vec![
            step("s2", "v-slot", 9_000),
            step("s2", "v-model", 13_000),
            step("s2", "hoist-static", 57_000),
        ]
    );
    // Timing observes: the pages are exactly the untimed ladder's.
    assert_eq!(run.pages, ladder_pages("src/App.vue", TEMPLATE));
}

#[test]
fn the_timings_export_as_a_schema_valid_profile_document() {
    let clock = squares();
    let run = ladder_run("src/App.vue", TEMPLATE, &clock);
    let profile = ladder_profile("analyze-sfc", &run.steps, &run.walks);
    assert_eq!(LADDER_WALK_KEY, "davinci.pass.walk");
    assert_eq!(
        schema_check::validate(&load_schema(), &profile, "$"),
        Ok(())
    );

    // Percentiles are the exporter's histogram-bucket bounds, so a single
    // sample reports its bucket's upper edge there and its exact value in
    // total/self/min/max.
    let span = |key: &str, pass: &str, stage: &str, nanos: u64, bucket: u64| {
        serde_json::json!({
            "key": key,
            "count": 1,
            "wall_ns": {
                "total": nanos, "self": nanos, "min": nanos, "max": nanos,
                "p50": bucket, "p95": bucket, "p99": bucket,
            },
            "alloc": null,
            "attribution": { "stage": stage, "pass": pass, "block": "template" },
        })
    };
    let step = |pass: &str, stage: &str, nanos: u64, bucket: u64| {
        span(LADDER_STEP_KEY, pass, stage, nanos, bucket)
    };
    let walk =
        |pass: &str, nanos: u64, bucket: u64| span(LADDER_WALK_KEY, pass, "s2", nanos, bucket);
    // Ranked by total wall time, descending, ties by key (the exporter's
    // contract): a lone pass's walk ties its step and sorts first.
    assert_eq!(
        profile,
        serde_json::json!({
            "schema_version": 1,
            "tool": "vize",
            "tool_version": env!("CARGO_PKG_VERSION"),
            "command": "analyze-sfc",
            "budget": { "max_spans": 512, "max_counters": 256 },
            "truncation": { "dropped_spans": 0, "dropped_counters": 0 },
            "spans": [
                walk("hoist-static", 57_000, 64_000),
                step("lower", "s3", 25_000, 32_000),
                step("template-complexity", "s2", 21_000, 32_000),
                step("hoist-static", "s2", 17_000, 32_000),
                walk("v-model", 13_000, 16_000),
                step("v-model", "s2", 13_000, 16_000),
                walk("v-slot", 9_000, 16_000),
                step("v-slot", "s2", 9_000, 16_000),
                step("lower", "s2", 5_000, 8_000),
                step("parse", "s1", 1_000, 1_000),
            ],
            "counters": [],
            "allocation": null,
        })
    );
}
