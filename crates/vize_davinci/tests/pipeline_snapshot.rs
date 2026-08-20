//! TS-17, established (P2-4): folio in -> pipeline -> **full normalized
//! folio** snapshot out, with targeted structural asserts as supplements
//! only (assurance §4).
//!
//! There are no landed optimization passes until P2-9, so the harness runs
//! the pipeline machinery with no-op bodies over a committed croquis
//! fixture: what is established here is the *shape* every later pass test
//! reuses - parse the stage artifact, run a classified plan through
//! `run_pipeline` under the budget observer, snapshot the artifact with
//! `assert_folio_snapshot!` (the printer), and pin the walk accounting as
//! the structural supplement. The binary-level twin drives the same fixture
//! through `davinci-opt --pipeline` on stdin.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use vize_davinci::assert_folio_snapshot;
use vize_davinci::folio::{Folio, FolioMode, croquis::CroquisFolio};
use vize_davinci::pass::{
    BudgetObserver, Fusability, PassDesc, PassKind, Pipeline, Preserved, run_pipeline,
};

/// A pass-shaped plan: two fusable no-ops sharing a walk, then a mandatory
/// barrier - so the walk accounting below actually exercises fusion.
const NORMALIZE: PassDesc = PassDesc::new(
    "normalize",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);
const FOLD: PassDesc = PassDesc::new(
    "fold",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);
const CHECK: PassDesc = PassDesc::new(
    "check",
    PassKind::MandatoryDiagnostic,
    Fusability::Barrier,
    Preserved::ALL,
);
const PASSES: &[PassDesc] = &[NORMALIZE, FOLD, CHECK];
const PLAN: Pipeline = Pipeline::new("croquis", PASSES);

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("croquis")
        .join("props-destructure.folio")
}

#[test]
fn a_pipeline_run_snapshots_the_full_normalized_folio() {
    let text = std::fs::read_to_string(fixture()).expect("committed folio reads");
    let folio = CroquisFolio::parse(&text).expect("committed folio parses");

    let mut budget = BudgetObserver::new();
    run_pipeline(&PLAN, &mut budget, |_event| Ok(())).expect("a no-op pass body cannot fail");

    // The oracle: the full normalized folio after the pipeline ran.
    assert_folio_snapshot!(folio);

    // Structural supplements: the plan's walk accounting, pinned through
    // the budget observer's own derived folio page so the run's counts are
    // themselves a TS-16 artifact.
    assert_eq!(
        budget.print_to_string(FolioMode::Full).as_str(),
        "[budget-observer]\nwalks=2\npasses=3\nanalyses=0\npipelines=1\nfailures=0\n\n"
    );
}

#[test]
fn the_binary_pipeline_emits_the_same_normalized_folio() {
    let text = std::fs::read_to_string(fixture()).expect("committed folio reads");
    let mut child = Command::new(env!("CARGO_BIN_EXE_davinci-opt"))
        .args(["--pipeline", "croquis(normalize,fold,check)"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("davinci-opt spawns");
    child
        .stdin
        .as_mut()
        .expect("stdin is piped")
        .write_all(text.as_bytes())
        .expect("stdin accepts the folio");
    drop(child.stdin.take());
    let output = child.wait_with_output().expect("davinci-opt exits");

    assert_eq!(output.status.code(), Some(0));
    // The committed fixture is canonical and the passes are no-ops, so the
    // full normalized folio on stdout is the input byte-for-byte.
    assert_eq!(
        core::str::from_utf8(&output.stdout).expect("stdout is UTF-8"),
        text
    );
    // All three names bind as fusable no-ops in the binary (no pass
    // catalogue until P2-9), so the plan fuses to a single walk there.
    assert_eq!(
        core::str::from_utf8(&output.stderr).expect("stderr is UTF-8"),
        "davinci-opt: pipeline croquis(normalize,fold,check): walks=1 passes=3\n"
    );
}
