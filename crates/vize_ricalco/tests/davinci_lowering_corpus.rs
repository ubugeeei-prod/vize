//! Davinci P2-8 corpus-runnable entry — the P1-6/P1-7 lane shape.
//!
//! Compiled only with the `davinci-differential` feature (`[[test]]
//! required-features` in Cargo.toml). Runs the committed battery with its
//! exact-pinned scope and census (the same numbers the plain suite pins,
//! so a feature-wiring regression fails loudly in both lanes), then,
//! with `VIZE_DAVINCI_DIFFERENTIAL_CORPUS=<dir>`, additionally sweeps
//! every `.vue` file under `<dir>`: the **whole file's bytes** go
//! through the S1 parser (SFC block markup as markup, script/style as
//! raw text — the TS-19 lane's framing) and then through [`lower`],
//! asserting the full soundness oracle per file: id accounting,
//! Canonical-rigor verification, side-table resolution, and the folio
//! round-trip. Every file lowers or diagnoses; none may panic.
//!
//! Run:
//!
//! ```text
//! VIZE_DAVINCI_DIFFERENTIAL_CORPUS=tests/_fixtures/_git \
//!     cargo test -p vize_ricalco --features davinci-differential \
//!     --test davinci_lowering_corpus -- --nocapture
//! ```

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use davinci_test_support::corpus::{CorpusScope, corpus_root_from_env};
use davinci_test_support::surface_fixture as battery;
use support::{assert_sound, with_lowered};

fn collect_vue_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    let mut children: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    children.sort();
    for child in children {
        if child.is_dir() {
            // node_modules trees repeat the same shipped sources; the
            // sweep targets project code.
            if child.file_name().is_some_and(|name| name == "node_modules") {
                continue;
            }
            collect_vue_files(&child, out);
        } else if child.extension().is_some_and(|ext| ext == "vue") {
            out.push(child);
        }
    }
}

#[test]
fn lowering_corpus_is_total() {
    // -- committed battery, exact-pinned scope and census --------------
    assert_eq!(battery::WELL_FORMED.len(), 16);
    assert_eq!(battery::MALFORMED.len(), 26);
    let mut ops = 0u64;
    let mut diagnostics = 0usize;
    let mut provenance = 0usize;
    let mut scopes = 0usize;
    for fixture in battery::WELL_FORMED.iter().chain(battery::MALFORMED) {
        assert_sound(fixture.source, fixture.name);
        with_lowered(fixture.source, |lowered, _folio| {
            ops += u64::from(lowered.op_count);
            diagnostics += lowered.diagnostics.len();
            provenance += lowered.provenance.len();
            scopes += lowered.scopes.len();
        });
    }
    // Re-pinned at the element/binding-family installment (see
    // `lowering_battery.rs` for the movement's account).
    assert_eq!(
        (ops, diagnostics, provenance, scopes),
        (83, 28, 101, 1),
        "battery census moved: re-pin in both lanes deliberately"
    );

    // -- optional corpus sweep -----------------------------------------
    let Some(corpus_root) =
        corpus_root_from_env(env!("CARGO_MANIFEST_DIR")).unwrap_or_else(|error| panic!("{error}"))
    else {
        eprintln!("VIZE_DAVINCI_DIFFERENTIAL_CORPUS unset: committed battery only");
        return;
    };
    assert!(
        corpus_root.path().is_dir(),
        "VIZE_DAVINCI_DIFFERENTIAL_CORPUS must name a directory: {}",
        corpus_root.path().display()
    );
    eprintln!(
        "davinci lowering corpus scope: {} closure_evidence={}",
        corpus_root.scope().label(),
        matches!(corpus_root.scope(), CorpusScope::ClosureEvidence)
    );
    let mut files = Vec::new();
    collect_vue_files(corpus_root.path(), &mut files);
    assert!(
        !files.is_empty(),
        "corpus sweep found no .vue files under {}",
        corpus_root.path().display()
    );
    let mut checked = 0u64;
    let mut with_diagnostics = 0u64;
    for file in &files {
        let Ok(source) = fs::read_to_string(file) else {
            continue;
        };
        let context = file.to_string_lossy();
        assert_sound(&source, context.as_ref());
        with_lowered(&source, |lowered, _folio| {
            if !lowered.diagnostics.is_empty() {
                with_diagnostics += 1;
            }
        });
        checked += 1;
    }
    eprintln!(
        "davinci lowering corpus sweep: files={} checked={} with_diagnostics={}",
        files.len(),
        checked,
        with_diagnostics
    );
}
