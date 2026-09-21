//! TS-17 for the template-complexity analysis (Davinci P4-9a): committed
//! fixture in, full transform pipeline out, **full normalized folio**
//! snapshot plus the full printed breakdown. An analysis pass moves no
//! surface, so the folio is exactly what the lowering built (the
//! fact-not-mutation proof); the breakdown snapshot is the product. The
//! structural supplements pin the fusion (the pass adds a pass, not a
//! walk), the pipeline product equal to a standalone run, and the empty
//! diagnostics channel (assurance §4).

// The shared `support` oracle builds std strings for its span checks.
#![allow(clippy::disallowed_types)]

mod support;

use std::path::{Path, PathBuf};

use vize_davinci::folio::{Folio, FolioMode};
use vize_s1_to_s2::pass::cfg::{self, print_facts};

use support::{assert_transformed_sound, with_transformed};

fn fixture(name: &str) -> vize_s0::String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("complexity")
        .join(name);
    let text = std::fs::read_to_string(path).expect("committed fixture reads");
    vize_s0::String::from(text.as_str())
}

/// Snapshot one fixture under explicit names (`<stem>_folio`,
/// `<stem>_breakdown`); returns `(cyclomatic, cognitive)` for the pins.
#[allow(clippy::disallowed_macros)]
fn snapshot(stem: &str, budget_text: &str) -> (u32, u32) {
    let name = format!("{stem}.vue");
    let source = fixture(&name);
    let totals = with_transformed(&source, |lowered, folio, facts, budget| {
        insta::assert_snapshot!(
            format!("{stem}_folio"),
            folio.print_to_string(FolioMode::Full).as_str()
        );
        let complexity = facts
            .complexity
            .as_ref()
            .expect("the full plan runs the optional analysis");
        // The pipeline product is the standalone pass's product.
        assert_eq!(*complexity, cfg::run(lowered));
        insta::assert_snapshot!(
            format!("{stem}_breakdown"),
            print_facts(complexity, &source).as_str()
        );
        assert_eq!(
            budget.print_to_string(FolioMode::Full).as_str(),
            budget_text
        );
        // An analysis pass emits no diagnostics — `Optional`'s ground.
        assert_eq!(lowered.diagnostics, vec![]);
        (complexity.cyclomatic, complexity.cognitive)
    });
    assert_transformed_sound(&source, &name);
    totals
}

#[test]
fn the_constructs_fixture_snapshots_the_folio_and_the_breakdown() {
    // Three walks for four passes: `v-slot` and `v-model` are lone
    // barriers, and `hoist-static` + `template-complexity` share one.
    let totals = snapshot(
        "constructs",
        "[budget-observer]\nwalks=3\npasses=4\nanalyses=0\npipelines=1\nfailures=0\n\n",
    );
    assert_eq!(totals, (15, 18));
}

#[test]
fn the_dashboard_fixture_snapshots_the_folio_and_the_breakdown() {
    let totals = snapshot(
        "dashboard",
        "[budget-observer]\nwalks=3\npasses=4\nanalyses=0\npipelines=1\nfailures=0\n\n",
    );
    assert_eq!(totals, (13, 18));
}
