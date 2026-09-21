//! TS-31 source-map coverage measurement (Davinci P3-9).
//!
//! Compiles the TS-31 fixture battery with source maps on for every backend,
//! decodes the maps with an independent decoder, classifies every authored
//! anchor, and compares the measurement with the committed report at
//! `davinci-road/plan/ts31-sourcemap-coverage.json`. The budget comparison
//! lives in `tests/tooling/davinci-sourcemap-coverage.test.ts`, which reads
//! that report beside `davinci-road/plan/budgets.toml`.
//!
//! Regenerate the report after an intentional emitter change with
//! `VIZE_UPDATE_TS31_REPORT=1 cargo test -p vize_atelier_sfc --test davinci_ts31_sourcemap`.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod davinci_ts31_sourcemap {
    pub mod battery;
    pub mod compile;
    pub mod decode;
    pub mod measure;
}

use std::path::PathBuf;

use davinci_ts31_sourcemap::battery::{self, Backend, Fixture};
use davinci_ts31_sourcemap::measure::{self, Rows};
use davinci_ts31_sourcemap::{compile, decode};

const TEMPLATE_BACKENDS: [Backend; 3] = [Backend::Dom, Backend::Vapor, Backend::Ssr];
const UPDATE_ENV: &str = "VIZE_UPDATE_TS31_REPORT";

fn report_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../davinci-road/plan/ts31-sourcemap-coverage.json")
}

fn measure_fixture(rows: &mut Rows, backend: Backend, fixture: &Fixture, filename: &str) {
    let compiled = compile::compile(backend, fixture);
    let map = compiled
        .map
        .as_ref()
        .expect("source maps on always attach a map");
    let segments = decode::decode(map, &compiled.generated, &fixture.source, filename);
    for anchor in &fixture.anchors {
        let status = measure::classify(anchor, backend, &compiled.generated, &segments);
        rows.record(backend, anchor.category, &anchor.id, status);
    }
}

fn measure_battery() -> serde_json::Value {
    let templates = battery::load("templates.toml");
    let sfcs = battery::load("sfc.toml");
    let sfc_rewrites = battery::load("sfc-rewrites.toml");
    let mut rows = Rows::default();
    for backend in TEMPLATE_BACKENDS {
        for fixture in &templates {
            measure_fixture(&mut rows, backend, fixture, compile::TEMPLATE_FILENAME);
        }
    }
    for backend in [Backend::Sfc, Backend::LegacySfc] {
        for fixture in &sfcs {
            measure_fixture(&mut rows, backend, fixture, compile::SFC_FILENAME);
        }
    }
    // Rewritten statements are measured on the structured SFC map only.
    for fixture in &sfc_rewrites {
        measure_fixture(&mut rows, Backend::Sfc, fixture, compile::SFC_FILENAME);
    }
    let rows = rows
        .0
        .iter()
        .map(|(key, row)| (key.clone(), row.to_json()))
        .collect::<serde_json::Map<_, _>>();
    serde_json::json!({
        "suite": "TS-31",
        "command": "cargo test -p vize_atelier_sfc --test davinci_ts31_sourcemap",
        "battery": {
            "templates": templates.len(),
            "sfcs": sfcs.len(),
            "sfc_rewrites": sfc_rewrites.len(),
        },
        "rows": rows,
    })
}

#[test]
fn ts31_measurement_matches_committed_report() {
    let measured = measure_battery();
    let path = report_path();
    if std::env::var(UPDATE_ENV).as_deref() == Ok("1") {
        let mut text = serde_json::to_string_pretty(&measured).expect("report serializes");
        text.push('\n');
        std::fs::write(&path, text).expect("write TS-31 report");
    }
    let committed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("TS-31 report is committed"))
            .expect("TS-31 report is JSON");
    assert_eq!(
        committed, measured,
        "TS-31 report is stale; rerun with {UPDATE_ENV}=1 and review the diff"
    );
}
