//! Identical public Doctor filter probe compiled against exact base and head.

#![expect(
    clippy::disallowed_macros,
    reason = "the identical public API probe crosses dependency alias migrations"
)]

mod fixtures;

use std::{hint::black_box, io, time::Instant};

use fixtures::scenario;

fn main() -> io::Result<()> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "doctor-filter-sample.json".into());
    let mut rows = Vec::new();
    for kind in [
        "rule-literal",
        "rule-regex",
        "path-prefix",
        "path-suffix",
        "path-regex",
    ] {
        for patterns in [1, 4, 16, 64, 256] {
            let (spec, findings, expected) = scenario(kind, patterns);
            let compile_start = Instant::now();
            let filter = spec.compile().map_err(io::Error::other)?;
            let compile_ns = compile_start.elapsed().as_nanos();
            let verdicts: Vec<bool> = findings
                .iter()
                .map(|finding| filter.matches(finding))
                .collect();
            if verdicts != expected {
                return Err(io::Error::other(
                    "Doctor filter changed planted hit/miss verdicts",
                ));
            }
            let iterations = 512;
            let start = Instant::now();
            let mut matched = 0;
            for _ in 0..iterations {
                for finding in &findings {
                    matched += usize::from(black_box(&filter).matches(black_box(finding)));
                }
            }
            let elapsed_ns = start.elapsed().as_nanos();
            if black_box(matched) != iterations * expected.iter().filter(|hit| **hit).count() {
                return Err(io::Error::other(
                    "timed Doctor filter lost planted findings",
                ));
            }
            rows.push(serde_json::json!({
                "id": format!("{kind}/{patterns}"), "kind": kind,
                "patterns": patterns, "candidates": findings.len(), "iterations": iterations,
                "matched": matched, "verdicts": verdicts, "elapsed_ns": elapsed_ns,
                "compile_ns": compile_ns,
            }));
        }
    }
    let report = serde_json::json!({ "schema": 1,
        "sha": std::env::var("MEASURE_SHA").unwrap_or_default(),
        "side": std::env::var("MEASURE_SIDE").unwrap_or_default(), "rows": rows });
    std::fs::write(output, serde_json::to_vec_pretty(&report)?)
}
