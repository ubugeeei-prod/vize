//! P3-10 try-measure-commit extraction over the documented placement fixture.
//!
//! Expected metrics are computed by hand from the cost model in
//! `extract/measure.rs`. Fixture payload: 45 operand bytes. Canonical plan:
//! seven effect units (7 x 21 = 147 bytes), keys `save`, `msg` x3 and `ok`
//! (5 edges), and update path 1 + 1 + 1 + (1 + 2 re-rendered) + 1 = 7.

use super::fixture::{Build, JS, LIT, fixture};
use vize_davinci::folio::remarks::RemarkLog;
use vize_davinci::folio::{Folio, FolioMode};
use vize_davinci::pass::{
    Fusability, NoObserver, PassKind, Preserved, RemarkCollector, RemarkCounter,
};
use vize_impeto::extract::{
    Decision, DecisionKind, Delta, EXTRACT, Extraction, Metric, Metrics, OptTier, Reason, extract,
};
use vize_impeto::op::{OpId, OpKind};
use vize_impeto::optimize::{OPTIMIZE, optimize};
use vize_impeto::placement::{ANNOTATE, Placement, annotate};
use vize_impeto::verify::verify;
use vize_s0::{Allocator, Span};

const CANONICAL: Metrics = Metrics {
    emitted_size: 192,
    reactive_edges: 5,
    update_path: 7,
};

fn decision(op: u32, span: (u32, u32), placement: Placement, reason: Reason) -> Decision {
    Decision {
        op: OpId::new(op),
        span: Span::new(span.0, span.1),
        placement,
        kind: if reason == Reason::Committed {
            DecisionKind::Applied
        } else {
            DecisionKind::Missed
        },
        reason,
        delta: Delta::default(),
        budget_left: 0,
    }
}

fn with(decision: Decision, delta: (i64, i64, i64), budget_left: u32) -> Decision {
    Decision {
        delta: Delta {
            emitted_size: delta.0,
            reactive_edges: delta.1,
            update_path: delta.2,
        },
        budget_left,
        ..decision
    }
}

#[test]
fn o1_commits_the_group_and_hoist_and_measures_the_cache_as_a_size_regression() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    annotate(&mut build.program);
    let extraction = extract(&mut build.program, OptTier::O1);

    let cache = decision(
        1,
        (5, 15),
        Placement::Cache,
        Reason::Regressed(Metric::EmittedSize),
    );
    let group = decision(3, (30, 40), Placement::Group, Reason::Committed);
    let hoist = decision(5, (45, 75), Placement::Hoist, Reason::Committed);
    assert_eq!(
        extraction,
        Extraction {
            tier: OptTier::O1,
            candidate_budget: 8,
            budget_left: 5,
            before: CANONICAL,
            after: Metrics {
                emitted_size: 158,
                reactive_edges: 4,
                update_path: 6,
            },
            decisions: vec![
                with(cache, (6, -1, -1), 7),
                with(group, (-21, -1, 0), 6),
                with(hoist, (-13, 0, -1), 5),
            ],
        }
    );
    let chosen: Vec<(OpId, Placement)> = build
        .program
        .placements
        .iter()
        .map(|record| (record.op, record.chosen))
        .collect();
    assert_eq!(
        chosen,
        [
            (OpId::new(1), Placement::Inline),
            (OpId::new(3), Placement::Group),
            (OpId::new(5), Placement::Hoist),
        ]
    );
    assert_eq!(verify(&build.program), []);
}

#[test]
fn o0_spends_no_budget_and_keeps_the_canonical_plan() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    annotate(&mut build.program);
    let extraction = extract(&mut build.program, OptTier::O0);

    assert_eq!(
        extraction.decisions,
        [
            decision(1, (5, 15), Placement::Cache, Reason::BudgetExhausted),
            decision(3, (30, 40), Placement::Group, Reason::BudgetExhausted),
            decision(5, (45, 75), Placement::Hoist, Reason::BudgetExhausted),
        ]
    );
    assert_eq!(
        (extraction.before, extraction.after),
        (CANONICAL, CANONICAL)
    );
}

#[test]
fn extraction_restarts_from_the_canonical_plan_and_is_deterministic() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    annotate(&mut build.program);
    let first = extract(&mut build.program, OptTier::O3);
    let second = extract(&mut build.program, OptTier::O3);
    assert_eq!(second, first);
    assert_eq!(first.before, CANONICAL);
    assert_eq!(first.budget_left, 125);
}

#[test]
fn the_budget_decrements_per_measured_trial_and_breaks_later_groups() {
    let arena = Allocator::default();
    let mut build = Build::new(&arena, (0, 110));
    for id in 0..11u32 {
        build.text(id, 0, (id * 10, id * 10 + 10), (JS, "count"), false);
    }
    annotate(&mut build.program);
    let extraction = extract(&mut build.program, OptTier::O1);

    let reasons: Vec<(OpId, Reason, u32)> = extraction
        .decisions
        .iter()
        .map(|decision| (decision.op, decision.reason, decision.budget_left))
        .collect();
    let mut expected: Vec<(OpId, Reason, u32)> = (1..=8)
        .map(|op| (OpId::new(op), Reason::Committed, 8 - op))
        .collect();
    expected.push((OpId::new(9), Reason::BudgetExhausted, 0));
    expected.push((OpId::new(10), Reason::NotContiguous, 0));
    assert_eq!(reasons, expected);
    assert_eq!(verify(&build.program), []);
}

#[test]
fn a_lone_static_node_costs_more_to_hoist_than_it_saves() {
    let arena = Allocator::default();
    let mut build = Build::new(&arena, (0, 40));
    build.op(0, OpKind::If, 0, (0, 40), true);
    build.region(1, 0, 0, (10, 30));
    build.condition(0, 1, "ok");
    build.element(1, "br", 1, (10, 30), true);
    annotate(&mut build.program);
    let extraction = extract(&mut build.program, OptTier::O3);

    let hoist = decision(
        1,
        (10, 30),
        Placement::Hoist,
        Reason::Regressed(Metric::EmittedSize),
    );
    assert_eq!(extraction.decisions, [with(hoist, (8, 0, 0), 127)]);
}

#[test]
fn an_enclosing_hoist_subsumes_an_inner_candidate_without_spending_budget() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    build.element(8, "b", 3, (60, 70), true);
    build.record(5, &[Placement::Inline, Placement::Hoist], None);
    build.record(8, &[Placement::Inline, Placement::Hoist], None);
    let extraction = extract(&mut build.program, OptTier::O1);

    let outer = decision(5, (45, 75), Placement::Hoist, Reason::Committed);
    let inner = decision(8, (60, 70), Placement::Hoist, Reason::Subsumed);
    assert_eq!(
        extraction.decisions,
        [with(outer, (-34, 0, -2), 7), with(inner, (0, 0, 0), 7)]
    );
    assert_eq!(verify(&build.program), []);
}

const REMARKS: &str = "\
[remarks]

[remarks.entries]
s3.extract-placements missed cache @5:15 reason=\"regressed-emitted-size\" emitted-size=6 reactive-edges=-1 update-path=-1 budget-left=7
s3.extract-placements applied group @30:40 reason=\"committed\" emitted-size=-21 reactive-edges=-1 update-path=0 budget-left=6
s3.extract-placements applied hoist @45:75 reason=\"committed\" emitted-size=-13 reactive-edges=0 update-path=-1 budget-left=5

";

#[test]
fn the_pipeline_attributes_one_structured_remark_per_decision() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    let mut collector = RemarkCollector::new();
    let extraction =
        optimize(&mut build.program, OptTier::O1, &mut collector).expect("closed pipeline");
    let page = RemarkLog::new(collector.finish()).print_to_string(FolioMode::Full);
    assert_eq!(page.as_str(), REMARKS);

    let mut detached = fixture(&arena, (LIT, "hi"));
    assert_eq!(
        optimize(&mut detached.program, OptTier::O1, &mut NoObserver),
        Ok(extraction)
    );
    let mut counted = fixture(&arena, (LIT, "hi"));
    let mut counter = RemarkCounter::new();
    optimize(&mut counted.program, OptTier::O1, &mut counter).expect("closed pipeline");
    assert_eq!(
        counter,
        RemarkCounter {
            applied: 2,
            missed: 1,
            analysis: 0
        }
    );
}

#[test]
fn both_passes_are_optional_graph_preserving_barriers_of_one_s3_pipeline() {
    for (desc, name) in [
        (ANNOTATE, "annotate-placements"),
        (EXTRACT, "extract-placements"),
    ] {
        assert_eq!(
            (desc.name, desc.kind, desc.fusability, desc.preserved),
            (
                name,
                PassKind::Optional,
                Fusability::Barrier,
                Preserved::ALL
            )
        );
    }
    assert_eq!(
        (OPTIMIZE.stage, OPTIMIZE.passes),
        ("s3", &[ANNOTATE, EXTRACT][..])
    );
    assert_eq!(OPTIMIZE.group_count(), 2);
}
