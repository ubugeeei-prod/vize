//! TS-17 for the template-complexity analysis (Davinci P4-9a): committed
//! fixture in, full transform pipeline out, **full normalized folio**
//! snapshot plus the full printed breakdown. An analysis pass moves no
//! surface, so the folio is exactly what the lowering built (the
//! fact-not-mutation proof); the breakdown snapshot is the product. The
//! structural supplements pin the fusion (the pass adds a pass, not a
//! walk), the pipeline product equal to a standalone run, and the empty
//! diagnostics channel (assurance §4).

#![expect(clippy::expect_used, reason = "tests assert by panicking")]
#![expect(
    clippy::disallowed_macros,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

// The shared `support` oracle builds std strings for its span checks.

mod support;

use std::path::{Path, PathBuf};

use vize_l0::dump::{Dump, Mode as DumpMode};
use vize_l1_to_l2::pass::cfg::{self, print_facts};

use support::{assert_transformed_sound, with_transformed};

fn fixture(name: &str) -> vize_l0::String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("complexity")
        .join(name);
    let text = std::fs::read_to_string(path).expect("committed fixture reads");
    vize_l0::String::from(text.as_str())
}

/// Snapshot one fixture under explicit names (`<stem>_folio`,
/// `<stem>_breakdown`); returns `(cyclomatic, cognitive)` for the pins.
fn snapshot(stem: &str, budget_text: &str) -> (u32, u32) {
    let name = format!("{stem}.vue");
    let source = fixture(&name);
    let totals = with_transformed(&source, |lowered, folio, facts, budget| {
        insta::assert_snapshot!(
            format!("{stem}_folio"),
            folio.print_to_string(DumpMode::Full).as_str()
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
        assert_eq!(budget.print_to_string(DumpMode::Full).as_str(), budget_text);
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

#[test]
fn empty_regions_preserve_all_facts_and_surrounding_page_order() {
    use vize_l0::Span;
    use vize_l0::id::NodeId;
    use vize_l1_to_l2::pass::cfg::{ComplexityFacts, Contribution, DecisionKind};

    support::with_lowered("", |lowered, _folio| {
        assert_eq!(lowered.op_count, 0);
        assert_eq!(cfg::run(lowered), ComplexityFacts::empty());
    });

    // Empty children before, within and after real decisions must not mint ids
    // or change nesting. A nonempty branch and interpolation retain their rows.
    let source = r#"<p/><div v-if="ok"></div><span>{{ ready ? 1 : 2 }}</span><footer/>"#;
    support::with_lowered(source, |lowered, _folio| {
        // cfg::run also asserts complete shared PageWalk minted accounting.
        let facts = cfg::run(lowered);
        assert_eq!(lowered.op_count, 6);
        assert_eq!(
            (
                facts.cyclomatic,
                facts.cognitive,
                facts.unknown,
                facts.max_nesting
            ),
            (3, 2, 0, 1)
        );
        let authored_span = |text: &str| {
            let start = u32::try_from(source.find(text).expect("authored control exists"))
                .expect("short control offset fits");
            let length = u32::try_from(text.len()).expect("short control length fits");
            Span::new(start, start + length)
        };
        assert_eq!(
            facts.contributions.as_slice(),
            &[
                Contribution {
                    span: authored_span("ok"),
                    kind: DecisionKind::If,
                    op: NodeId::from_index(1),
                    nesting: 0,
                    cyclomatic: 1,
                    cognitive: 1,
                },
                Contribution {
                    span: authored_span("ready ? 1 : 2"),
                    kind: DecisionKind::Conditional,
                    op: NodeId::from_index(4),
                    nesting: 0,
                    cyclomatic: 1,
                    cognitive: 1,
                },
            ]
        );
    });
}
