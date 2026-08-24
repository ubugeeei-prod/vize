//! P2-3's law: **a fused group is reported as one walk, with its member passes
//! named.**
//!
//! An ordinary `#[test]` under `tests/`, so it runs in the default
//! `cargo test --workspace` lane rather than a feature-gated one — the
//! P1-5/P1-7 counter-law shape, deliberately not a lane somebody has to
//! remember to enable.
//!
//! The failure this pins is specific and quiet: an observer that opened a
//! measurement per pass would report three walks where the manager made one,
//! and the numbers would look plausible enough to be believed. So the law is
//! asserted as an equality between what the observer counted and what the
//! plan says, over pipelines whose grouping differs.

use vize_davinci::pass::observer::{AnalysisEvent, FailEvent, Pair, PassEvent, PassObserver};
use vize_davinci::pass::{
    BudgetObserver, Fusability, NoObserver, PassDesc, PassFailure, PassKind, Pipeline, Preserved,
    run_pipeline,
};
use vize_s0::{String, cstr};

const A: PassDesc = PassDesc::new(
    "alpha",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);
const B: PassDesc = PassDesc::new(
    "beta",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);
const C: PassDesc = PassDesc::new(
    "gamma",
    PassKind::Optional,
    Fusability::Fusable,
    Preserved::ALL,
);
const BAR: PassDesc = PassDesc::new(
    "barrier",
    PassKind::MandatoryLowering,
    Fusability::Barrier,
    Preserved::NONE,
);

/// Records the exact hook sequence, so ordering is asserted rather than assumed.
#[derive(Default)]
struct Trace {
    events: Vec<String>,
}

impl PassObserver for Trace {
    fn before_pipeline(&mut self, pipeline: &Pipeline) {
        self.events.push(cstr!("open {}", pipeline.stage));
    }
    fn after_pipeline(&mut self, pipeline: &Pipeline) {
        self.events.push(cstr!("close {}", pipeline.stage));
    }
    fn before_pass(&mut self, event: &PassEvent<'_>) {
        self.events.push(cstr!(
            "> {} walk={} entry={} exit={}",
            event.desc().name,
            event.group_index,
            event.is_group_entry(),
            event.is_group_exit()
        ));
    }
    fn after_pass(&mut self, event: &PassEvent<'_>) {
        self.events.push(cstr!("< {}", event.desc().name));
    }
    fn before_analysis(&mut self, event: &AnalysisEvent<'_>) {
        self.events.push(cstr!("analysis {}", event.name));
    }
    fn on_fail(&mut self, event: &FailEvent<'_>) {
        self.events.push(cstr!("fail {}", event.reason));
    }
}

fn run_ok<O: PassObserver>(pipeline: &Pipeline, observer: &mut O) {
    run_pipeline(pipeline, observer, |_event| Ok(())).expect("no step fails");
}

#[test]
fn a_fused_group_is_counted_as_one_walk_not_one_per_pass() {
    const PASSES: &[PassDesc] = &[A, B, C];
    const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);
    // The plan says one walk; three passes share it.
    const _: () = assert!(PIPELINE.group_count() == 1);

    let mut budget = BudgetObserver::new();
    run_ok(&PIPELINE, &mut budget);

    assert_eq!(budget.walks, 1, "three fused passes are one walk");
    assert_eq!(budget.passes, 3);
    assert_eq!(budget.pipelines, 1);
    assert_eq!(budget.failures, 0);
    assert_eq!(budget.fusion_ratio_hundredths(), Some(300));
}

#[test]
fn the_walk_count_equals_the_plans_group_count_for_every_shape() {
    const SHAPES: [&[PassDesc]; 6] = [
        &[],
        &[A],
        &[A, B, C],
        &[A, BAR, B],
        &[BAR, BAR],
        &[A, B, BAR, C, BAR],
    ];
    for passes in SHAPES {
        let pipeline = Pipeline::new("s2", passes);
        let mut budget = BudgetObserver::new();
        run_ok(&pipeline, &mut budget);
        assert_eq!(
            budget.walks as usize,
            pipeline.group_count(),
            "observed walks must equal the planned group count"
        );
        assert_eq!(budget.passes as usize, passes.len());
    }
}

#[test]
fn every_pass_names_the_group_it_shares_and_marks_the_walk_boundaries() {
    const PASSES: &[PassDesc] = &[A, B, BAR, C];
    const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);

    let mut trace = Trace::default();
    run_ok(&PIPELINE, &mut trace);

    assert_eq!(
        trace.events,
        vec![
            String::new("open s2"),
            String::new("> alpha walk=0 entry=true exit=false"),
            String::new("< alpha"),
            String::new("> beta walk=0 entry=false exit=true"),
            String::new("< beta"),
            String::new("> barrier walk=1 entry=true exit=true"),
            String::new("< barrier"),
            String::new("> gamma walk=2 entry=true exit=true"),
            String::new("< gamma"),
            String::new("close s2"),
        ],
        "hook order, walk indices and boundary flags are all part of the contract"
    );
}

#[test]
fn group_members_names_every_pass_sharing_the_walk() {
    const PASSES: &[PassDesc] = &[A, B, BAR, C];
    const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);

    let mut seen: Vec<Vec<&str>> = Vec::new();
    run_pipeline(&PIPELINE, &mut NoObserver, |event| {
        if event.is_group_entry() {
            seen.push(event.group_members().iter().map(|d| d.name).collect());
        }
        Ok(())
    })
    .expect("no step fails");

    assert_eq!(
        seen,
        vec![vec!["alpha", "beta"], vec!["barrier"], vec!["gamma"]],
        "a walk must be able to name every pass fused into it"
    );
}

#[test]
fn a_failing_pass_fires_on_fail_and_suppresses_after_pipeline() {
    const PASSES: &[PassDesc] = &[A, B];
    const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);

    let mut trace = Trace::default();
    let result = run_pipeline(&PIPELINE, &mut trace, |event| {
        if event.desc().name == "beta" {
            Err(PassFailure::new("beta refused"))
        } else {
            Ok(())
        }
    });

    assert_eq!(result, Err(PassFailure::new("beta refused")));
    assert_eq!(
        trace.events,
        vec![
            String::new("open s2"),
            String::new("> alpha walk=0 entry=true exit=false"),
            String::new("< alpha"),
            String::new("> beta walk=0 entry=false exit=true"),
            String::new("fail beta refused"),
        ],
        "a failed run must not also report as finished, and the failing pass \
         must not report as completed"
    );
}

#[test]
fn a_failing_run_counts_the_failure_and_not_a_completed_pipeline() {
    const PASSES: &[PassDesc] = &[A, BAR];
    const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);

    let mut budget = BudgetObserver::new();
    let result = run_pipeline(&PIPELINE, &mut budget, |event| {
        if event.desc().name == "barrier" {
            Err(PassFailure::new("barrier refused"))
        } else {
            Ok(())
        }
    });

    assert_eq!(result, Err(PassFailure::new("barrier refused")));
    assert_eq!(budget.failures, 1);
    assert_eq!(
        budget.walks, 2,
        "both walks were entered before the failure"
    );
    assert_eq!(budget.passes, 2);
}

#[test]
fn composed_observers_both_see_every_hook() {
    const PASSES: &[PassDesc] = &[A, B, BAR];
    const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);

    let mut pair = Pair(BudgetObserver::new(), BudgetObserver::new());
    run_ok(&PIPELINE, &mut pair);

    assert_eq!(pair.0, pair.1, "composition must not favour either side");
    assert_eq!(pair.0.walks, 2);
    assert_eq!(pair.0.passes, 3);
}

#[test]
fn an_empty_pipeline_walks_nothing_but_still_opens_and_closes() {
    const PIPELINE: Pipeline = Pipeline::new("s2", &[]);
    let mut budget = BudgetObserver::new();
    run_ok(&PIPELINE, &mut budget);
    assert_eq!(budget.walks, 0);
    assert_eq!(budget.passes, 0);
    assert_eq!(budget.pipelines, 1);
}

#[test]
fn resetting_a_budget_observer_zeroes_every_counter() {
    const PASSES: &[PassDesc] = &[A, BAR];
    const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);
    let mut budget = BudgetObserver::new();
    run_ok(&PIPELINE, &mut budget);
    budget.reset();
    assert_eq!(budget, BudgetObserver::new());
}
