//! P3-13's channel law: **a remark reaches observers through the pass
//! manager, attributed to the pass that was running, and nowhere else.**
//!
//! Pinned as exact hook traces and exact recorded values: the remark hook
//! fires between the emitting pass's `before_pass` and `after_pass`, the
//! attribution comes from the event (a step cannot name another pass),
//! `Pair` forwards to exactly the members that consume remarks, and the
//! collector's canonical order is independent of emission order.

#![expect(clippy::expect_used, reason = "tests assert by panicking")]
#![expect(
    clippy::disallowed_types,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

use std::panic::{AssertUnwindSafe, catch_unwind};

use vize_davinci::pass::observer::{
    Pair, PassEvent, PassObserver, RecordedArg, RecordedRemark, RemarkArgValue,
};
use vize_davinci::pass::{
    BudgetObserver, Fusability, NoObserver, PassDesc, PassKind, Pipeline, Preserved, Remark,
    RemarkArg, RemarkCollector, RemarkCounter, RemarkKind, RemarkSink, run_pipeline_remarked,
};
use vize_s0::{Span, String, cstr};

const HOIST: PassDesc = PassDesc::new(
    "hoist",
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
const PASSES: &[PassDesc] = &[HOIST, FOLD, CHECK];
const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);

/// Records the hook sequence, remarks included.
#[derive(Default)]
struct Trace {
    events: Vec<String>,
}

impl PassObserver for Trace {
    const REMARKS: bool = true;

    fn before_pass(&mut self, event: &PassEvent<'_>) {
        self.events.push(cstr!("before {}", event.desc().name));
    }
    fn after_pass(&mut self, event: &PassEvent<'_>) {
        self.events.push(cstr!("after {}", event.desc().name));
    }
    fn on_remark(&mut self, event: &PassEvent<'_>, remark: &Remark<'_>) {
        self.events.push(cstr!(
            "remark {} {} {}",
            event.desc().name,
            remark.kind.as_str(),
            remark.name
        ));
    }
}

/// The fixture pass bodies: `hoist` emits two remarks out of span order,
/// `fold` one, `check` none.
fn emit_fixture<S: RemarkSink>(event: &PassEvent<'_>, remarks: &mut S) {
    if !remarks.enabled() {
        return;
    }
    match event.desc().name {
        "hoist" => {
            let tag = String::from("div");
            remarks.emit(&Remark::missed(
                "static-subtree",
                Span::new(20, 30),
                &[
                    RemarkArg::str("tag", tag.as_str()),
                    RemarkArg::str("blocker", "child"),
                ],
            ));
            remarks.emit(&Remark::applied(
                "static-subtree",
                Span::new(0, 40),
                &[RemarkArg::int("depth", -1), RemarkArg::bool("root", true)],
            ));
        }
        "fold" => remarks.emit(&Remark::analysis("folded", Span::new(5, 6), &[])),
        _ => {}
    }
}

fn run<O: PassObserver>(observer: &mut O) {
    run_pipeline_remarked(&PIPELINE, observer, |event, remarks| {
        emit_fixture(event, remarks);
        Ok(())
    })
    .expect("no step fails");
}

#[test]
fn remarks_fire_between_their_pass_hooks() {
    let mut trace = Trace::default();
    run(&mut trace);
    assert_eq!(
        trace.events,
        [
            "before hoist",
            "remark hoist missed static-subtree",
            "remark hoist applied static-subtree",
            "after hoist",
            "before fold",
            "remark fold analysis folded",
            "after fold",
            "before check",
            "after check",
        ]
    );
}

#[test]
fn remark_consumption_is_a_compile_time_property_of_the_observer() {
    const _: () = assert!(!NoObserver::REMARKS);
    const _: () = assert!(!BudgetObserver::REMARKS);
    const _: () = assert!(RemarkCounter::REMARKS);
    const _: () = assert!(RemarkCollector::REMARKS);
    const _: () = assert!(!Pair::<NoObserver, BudgetObserver>::REMARKS);
    const _: () = assert!(Pair::<NoObserver, RemarkCounter>::REMARKS);
    const _: () = assert!(Pair::<RemarkCounter, NoObserver>::REMARKS);

    // A detached run: the fixture body sees the channel disabled and
    // builds nothing; the budget still counts every pass.
    let mut budget = BudgetObserver::new();
    run(&mut budget);
    assert_eq!((budget.walks, budget.passes), (2, 3));
}

#[test]
fn a_pair_forwards_remarks_to_exactly_its_consuming_members() {
    let mut observers = Pair(
        Pair(BudgetObserver::new(), RemarkCounter::new()),
        Trace::default(),
    );
    run(&mut observers);
    let Pair(Pair(budget, counter), trace) = observers;
    assert_eq!((budget.walks, budget.passes), (2, 3));
    assert_eq!(
        counter,
        RemarkCounter {
            applied: 1,
            missed: 1,
            analysis: 1,
        }
    );
    assert_eq!(counter.total(), 3);
    assert_eq!(trace.events.len(), 9);
}

fn arg(key: &str, value: RemarkArgValue) -> RecordedArg {
    RecordedArg {
        key: String::from(key),
        value,
    }
}

fn recorded(
    pass: &str,
    kind: RemarkKind,
    name: &str,
    span: (u32, u32),
    args: Vec<RecordedArg>,
) -> RecordedRemark {
    RecordedRemark {
        stage: String::from("s2"),
        pass: String::from(pass),
        kind,
        name: String::from(name),
        span: Span::new(span.0, span.1),
        args,
    }
}

#[test]
fn the_collector_records_attribution_and_canonical_order() {
    let mut collector = RemarkCollector::new();
    run(&mut collector);
    assert_eq!(collector.len(), 3);
    // `hoist` emitted the inner span first; canonical order puts the outer
    // one (earlier start) first. `fold` stays after `hoist` although its
    // span starts earlier than the inner hoist remark: pass run position
    // outranks span.
    assert_eq!(
        collector.finish(),
        vec![
            recorded(
                "hoist",
                RemarkKind::Applied,
                "static-subtree",
                (0, 40),
                vec![
                    arg("depth", RemarkArgValue::Int(-1)),
                    arg("root", RemarkArgValue::Bool(true)),
                ],
            ),
            recorded(
                "hoist",
                RemarkKind::Missed,
                "static-subtree",
                (20, 30),
                vec![
                    arg("tag", RemarkArgValue::Str(String::from("div"))),
                    arg("blocker", RemarkArgValue::Str(String::from("child"))),
                ],
            ),
            recorded("fold", RemarkKind::Analysis, "folded", (5, 6), vec![]),
        ]
    );
}

#[test]
fn equal_starts_order_the_outer_span_first_then_by_name() {
    let mut collector = RemarkCollector::new();
    run_pipeline_remarked(&PIPELINE, &mut collector, |event, remarks| {
        if event.desc().name == "hoist" {
            remarks.emit(&Remark::applied("zeta", Span::new(4, 9), &[]));
            remarks.emit(&Remark::applied("beta", Span::new(4, 9), &[]));
            remarks.emit(&Remark::applied("alpha", Span::new(4, 5), &[]));
            remarks.emit(&Remark::missed("beta", Span::new(4, 9), &[]));
        }
        Ok(())
    })
    .expect("no step fails");
    let order: Vec<(String, RemarkKind, u32)> = collector
        .finish()
        .into_iter()
        .map(|remark| (remark.name, remark.kind, remark.span.end))
        .collect();
    assert_eq!(
        order,
        [
            (String::from("beta"), RemarkKind::Applied, 9),
            (String::from("beta"), RemarkKind::Missed, 9),
            (String::from("zeta"), RemarkKind::Applied, 9),
            (String::from("alpha"), RemarkKind::Applied, 5),
        ]
    );
}

#[test]
fn a_repeated_argument_key_is_a_pass_bug() {
    let mut collector = RemarkCollector::new();
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        run_pipeline_remarked(&PIPELINE, &mut collector, |_event, remarks| {
            remarks.emit(&Remark::applied(
                "dup",
                Span::new(0, 1),
                &[RemarkArg::int("n", 1), RemarkArg::int("n", 2)],
            ));
            Ok(())
        })
    }));
    let payload = outcome.expect_err("a duplicate key panics");
    let message = payload
        .downcast_ref::<std::string::String>()
        .map(std::string::String::as_str);
    assert_eq!(
        message,
        Some("remark `dup` from pass `hoist` repeats argument `n`")
    );
}

#[test]
fn every_kind_spells_and_reads_back_exactly() {
    for kind in RemarkKind::ALL {
        assert_eq!(RemarkKind::from_name(kind.as_str()), Some(kind));
    }
    assert_eq!(
        RemarkKind::ALL.map(RemarkKind::as_str),
        ["applied", "missed", "analysis"]
    );
    assert_eq!(RemarkKind::from_name("Applied"), None);
}
