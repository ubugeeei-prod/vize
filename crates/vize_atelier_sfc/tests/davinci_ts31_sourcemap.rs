//! TS-31 source-map coverage measurement (Davinci P3-9).
//!
//! Compiles the TS-31 fixture battery with source maps on for every live
//! backend, decodes the maps with an independent decoder, classifies every
//! authored anchor, and compares the measurement with the committed report at
//! `davinci-road/plan/ts31-sourcemap-coverage.json`. The legacy text-matching
//! recovery is deleted, so `legacy-sfc/text-matching-recovery` is not
//! remeasured: the harness copies that row's frozen anchor statuses out of
//! the committed report, including when `VIZE_UPDATE_TS31_REPORT=1` rewrites
//! the live rows. The budget comparison lives in
//! `tests/tooling/davinci-sourcemap-coverage.test.ts`.
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
/// Frozen statuses of the deleted text-matching recovery. Not remeasured.
const FROZEN_LEGACY_ROW: &str = "legacy-sfc/text-matching-recovery";

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

fn read_report(path: &std::path::Path) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(path).expect("TS-31 report is committed"))
        .expect("TS-31 report is JSON")
}

/// The deleted recovery's last measured row. A rewrite must copy it back
/// rather than dropping the floor the structured map is compared against.
fn frozen_legacy_row(committed: &serde_json::Value) -> serde_json::Value {
    let row = committed
        .get("rows")
        .and_then(|rows| rows.get(FROZEN_LEGACY_ROW))
        .cloned()
        .unwrap_or_else(|| panic!("TS-31 report is missing the frozen {FROZEN_LEGACY_ROW} row"));
    let anchors = row.get("anchors").and_then(|value| value.as_object());
    assert!(
        anchors.is_some_and(|anchors| !anchors.is_empty()),
        "frozen {FROZEN_LEGACY_ROW} row must keep its anchor statuses"
    );
    row
}

fn measure_battery(frozen_legacy: serde_json::Value) -> serde_json::Value {
    let templates = battery::load("templates.toml");
    let sfcs = battery::load("sfc.toml");
    let sfc_rewrites = battery::load("sfc-rewrites.toml");
    let mut rows = Rows::default();
    for backend in TEMPLATE_BACKENDS {
        for fixture in &templates {
            measure_fixture(&mut rows, backend, fixture, compile::TEMPLATE_FILENAME);
        }
    }
    for fixture in &sfcs {
        measure_fixture(&mut rows, Backend::Sfc, fixture, compile::SFC_FILENAME);
    }
    // Rewritten statements are measured on the structured SFC map only.
    for fixture in &sfc_rewrites {
        measure_fixture(&mut rows, Backend::Sfc, fixture, compile::SFC_FILENAME);
    }
    let mut rows = rows
        .0
        .iter()
        .map(|(key, row)| (key.clone(), row.to_json()))
        .collect::<serde_json::Map<_, _>>();
    let replaced = rows.insert(FROZEN_LEGACY_ROW.to_string(), frozen_legacy);
    assert!(
        replaced.is_none(),
        "the frozen legacy row must not be produced by a live measurement"
    );
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
    let path = report_path();
    let committed = read_report(&path);
    let measured = measure_battery(frozen_legacy_row(&committed));
    if std::env::var(UPDATE_ENV).as_deref() == Ok("1") {
        let mut text = serde_json::to_string_pretty(&measured).expect("report serializes");
        text.push('\n');
        std::fs::write(&path, text).expect("write TS-31 report");
    }
    assert_eq!(
        committed, measured,
        "TS-31 report is stale; rerun with {UPDATE_ENV}=1 and review the diff"
    );
}
