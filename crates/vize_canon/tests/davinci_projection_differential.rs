//! TS-25 lane for P4-5b: the S2 projection (an S4 target) against the current
//! Canon generator over the TS-40 fixture matrix.
//!
//! Generated text is free; what must agree is where an authored construct
//! lands. Slice 1 compares authored-anchor resolution: for every occurrence
//! of every matrix anchor, whether each side's `ProjectionMapping` resolves
//! it (a row or sub-span whose authored range covers it). The divergences
//! are an exact ledger (`davinci-road/plan/ts25-projection-divergence.txt`)
//! that may only shrink; P4-5b closes at an empty ledger plus the
//! `vize check` diagnostic-set comparison, which slice 1 does not run yet.
//! `VIZE_UPDATE_TS25_PROJECTION=1` rewrites the ledger for review.

use std::path::{Path, PathBuf};

use vize_canon::batch::{
    ImportRewriter, VueDocumentVirtualTsOptions, generate_vue_document_virtual_ts_with_options,
};
use vize_canon::projection::project_sfc;
use vize_canon::virtual_ts::{ProjectionMapping, VirtualTsOptions};
use vize_s0::{String, append, cstr};

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf()
}

struct Fixture {
    id: String,
    file: String,
    anchors: Vec<String>,
    crlf: bool,
    options_api: bool,
    legacy_vue2: bool,
}

fn fixtures() -> Vec<Fixture> {
    let matrix: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(
            workspace().join("tests/_fixtures/davinci-ts40-projection/matrix.json"),
        )
        .expect("TS-40 matrix"),
    )
    .expect("matrix JSON");
    matrix["fixtures"]
        .as_array()
        .expect("fixtures")
        .iter()
        .map(|fixture| Fixture {
            id: fixture["id"].as_str().unwrap().into(),
            file: fixture["file"].as_str().unwrap().into(),
            anchors: fixture["anchors"]
                .as_array()
                .map(|anchors| anchors.iter().map(|a| a.as_str().unwrap().into()).collect())
                .unwrap_or_default(),
            crlf: fixture["lineEnding"] == "crlf",
            options_api: fixture["optionsApi"] == true,
            legacy_vue2: fixture["legacyVue2"] == true,
        })
        .collect()
}

fn resolves(mapping: &ProjectionMapping, start: usize, end: usize) -> bool {
    mapping.spans().iter().any(|row| {
        (row.src_range.start <= start && end <= row.src_range.end)
            || row
                .sub_spans
                .iter()
                .any(|span| span.src_range.start <= start && end <= span.src_range.end)
    })
}

/// Every comparison and the divergent ones, in matrix order.
fn measure() -> (usize, String) {
    let mut comparisons = 0;
    let mut ledger = String::default();
    for fixture in fixtures() {
        let mut source: String = std::fs::read_to_string(workspace().join(fixture.file.as_str()))
            .expect("fixture source")
            .into();
        if fixture.crlf {
            source = source.replace("\r\n", "\n").replace('\n', "\r\n").into();
        }
        let current = generate_vue_document_virtual_ts_with_options(
            Path::new(fixture.file.as_str()),
            &source,
            &VirtualTsOptions::default(),
            &ImportRewriter::new(),
            false,
            VueDocumentVirtualTsOptions {
                options_api: fixture.options_api,
                legacy_vue2: fixture.legacy_vue2,
                experimental_patterned_template: false,
                preserve_event_navigation: true,
                dialect: Default::default(),
                preserve_missing_vue_diagnostics: true,
            },
        )
        .ok()
        .map(|document| document.mapping);
        let projected = project_sfc(&source).map(|projection| projection.mapping);
        for anchor in &fixture.anchors {
            for (start, _) in source.match_indices(anchor.as_str()) {
                let end = start + anchor.len();
                comparisons += 1;
                let old = current.as_ref().is_some_and(|m| resolves(m, start, end));
                let new = projected.as_ref().is_some_and(|m| resolves(m, start, end));
                if old != new {
                    append!(
                        ledger,
                        "{} {anchor}@{start} current={old} s2={new}\n",
                        fixture.id
                    );
                }
            }
        }
    }
    (comparisons, ledger)
}

/// Plain-suite coverage witness: the lane compares every anchor occurrence
/// of the matrix, never silently fewer.
#[test]
fn the_differential_compares_every_matrix_anchor() {
    assert_eq!(measure().0, 57);
}

#[cfg(feature = "davinci-differential")]
#[test]
fn s2_projection_divergence_matches_the_ledger() {
    let (comparisons, ledger) = measure();
    let path = workspace().join("davinci-road/plan/ts25-projection-divergence.txt");
    let header = cstr!(
        "# TS-25 P4-5b authored-anchor divergence: {comparisons} comparisons\n\
         # `vize check` diagnostic-set comparison: not run (P4-5b slice 2)\n"
    );
    let measured = cstr!("{header}{ledger}");
    if std::env::var_os("VIZE_UPDATE_TS25_PROJECTION").is_some() {
        std::fs::write(&path, measured.as_str()).expect("write ledger");
    }
    let committed = std::fs::read_to_string(&path).expect("committed ledger");
    assert_eq!(
        measured.as_str(),
        committed,
        "the ledger may only change with review"
    );
}
