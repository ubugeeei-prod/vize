//! TS-17, established (P2-4): folio in -> pipeline -> **full normalized
//! folio** snapshot out, with targeted structural asserts as supplements
//! only (assurance §4).
//!
//! There are no landed optimization passes until P2-9, so the harness runs
//! the pipeline machinery with no-op bodies over a committed croquis
//! fixture: what is established here is the *shape* every later pass test
//! reuses - parse the stage artifact, run a classified plan through
//! `run_pipeline` under the budget observer, snapshot the artifact with
//! `assert_dump_snapshot!` (the printer), and pin the walk accounting as
//! the structural supplement. The retired host's catalogue-free no-op
//! binding is no longer a product route.

use std::path::{Path, PathBuf};

use vize_davinci::assert_dump_snapshot;
use vize_davinci::dump::croquis::Page as CroquisPage;
use vize_davinci::dump::{Dump, Mode as DumpMode};
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
    let folio = CroquisPage::parse(&text).expect("committed folio parses");

    let mut budget = BudgetObserver::new();
    run_pipeline(&PLAN, &mut budget, |_event| Ok(())).expect("a no-op pass body cannot fail");

    // The oracle: the full normalized folio after the pipeline ran.
    assert_dump_snapshot!(folio);

    // Structural supplements: the plan's walk accounting, pinned through
    // the budget observer's own derived folio page so the run's counts are
    // themselves a TS-16 artifact.
    assert_eq!(
        budget.print_to_string(DumpMode::Full).as_str(),
        "[budget-observer]\nwalks=2\npasses=3\nanalyses=0\npipelines=1\nfailures=0\n\n"
    );
}
