//! P3-10 try-measure-commit extraction over the documented placement fixture.
//!
//! Expected metrics are computed by hand from the cost model in
//! `extract/measure.rs`. Fixture payload: 45 operand bytes. Canonical plan:
//! seven effect units (7 x 21 = 147 bytes), keys `save`, `msg` x3 and `ok`
//! (5 edges), and update path 1 + 1 + 1 + (1 + 2 re-rendered) + 1 = 7.

use super::fixture::{Build, JS, LIT, fixture};
use vize_davinci::pass::observer::remark::CountingRemarkSink;
use vize_davinci::pass::{Fusability, PassKind, Preserved};
use vize_impeto::extract::{
    Decision, DecisionKind, Delta, EXTRACT, Extraction, Metric, Metrics, OptTier, Reason, extract,
};
use vize_impeto::op::{OpId, OpKind};
use vize_impeto::placement::{Placement, annotate};
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

#[test]
fn every_decision_becomes_one_remark() {
    let arena = Allocator::default();
    let mut build = fixture(&arena, (LIT, "hi"));
    annotate(&mut build.program);
    let mut sink = CountingRemarkSink::new();
    extract(&mut build.program, OptTier::O2).remark(&mut sink);
    assert_eq!(
        sink,
        CountingRemarkSink {
            applied: 2,
            missed: 1
        }
    );
}

#[test]
fn the_pass_is_an_optional_graph_preserving_barrier() {
    assert_eq!(EXTRACT.name, "impeto-extract");
    assert_eq!(EXTRACT.kind, PassKind::Optional);
    assert_eq!(EXTRACT.fusability, Fusability::Barrier);
    assert_eq!(EXTRACT.preserved, Preserved::ALL);
}
