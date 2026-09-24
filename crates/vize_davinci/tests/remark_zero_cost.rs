//! P3-13's zero-cost law: **with no remark consumer attached, a pass that
//! explains its decisions allocates exactly what one that does not would.**
//!
//! The fixture pass builds each remark's text argument on the heap (exactly
//! one allocation), guarded by the channel's compile-time `ENABLED`. Under
//! an observer that consumes no remarks the guarded path must vanish, so the
//! measured window allocates **exactly zero** - the P2-3 bench pair's
//! `allocs = 0` instrument (`davinci_harness`'s counting allocator), applied
//! to the remark path. The attached run is the control: the same body under
//! a (non-allocating) counter allocates exactly one label per pass, so a
//! zero in the detached runs is the observer's absence, not a guard that
//! hides the work everywhere.
//!
//! This binary owns the process global allocator and holds a single test,
//! so no concurrent test pollutes the counters.

#![expect(clippy::expect_used, reason = "tests assert by panicking")]

use davinci_harness::alloc::{CountingAllocator, mark_installed, measure};
use vize_davinci::pass::{
    BudgetObserver, Fusability, NoObserver, Pair, PassDesc, PassEvent, PassKind, PassObserver,
    Pipeline, Preserved, Remark, RemarkArg, RemarkCounter, RemarkSink, TimingObserver,
    run_pipeline_remarked,
};
use vize_s0::Span;

#[global_allocator]
static ALLOCATOR: CountingAllocator<std::alloc::System> = CountingAllocator::system();

const HOIST: PassDesc = PassDesc::new(
    "hoist",
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
const PASSES: &[PassDesc] = &[HOIST, HOIST, CHECK];
const PIPELINE: Pipeline = Pipeline::new("s2", PASSES);

/// A body that explains itself: one remark per pass, its text argument
/// built with exactly one heap allocation.
fn explaining_body<S: RemarkSink>(event: &PassEvent<'_>, remarks: &mut S) {
    if S::ENABLED {
        let label = vec![b'a' + event.pass_index as u8; 64];
        let label = core::str::from_utf8(&label).expect("ASCII label");
        remarks.emit(&Remark::analysis(
            "visited",
            Span::new(0, 1),
            &[RemarkArg::str("label", label)],
        ));
    }
}

fn run<O: PassObserver>(observer: &mut O) {
    run_pipeline_remarked(&PIPELINE, observer, |event, remarks| {
        explaining_body(event, remarks);
        Ok(())
    })
    .expect("no step fails");
}

#[test]
fn a_detached_remark_path_allocates_nothing() {
    mark_installed();

    let detached = measure(|| run(&mut NoObserver)).expect("counting allocator installed");
    assert_eq!(detached.calls, 0, "NoObserver: the remark path must vanish");

    // The observers every shipped driver attaches consume no remarks;
    // composing them must not switch the path back on.
    let mut shipped = Pair(TimingObserver::new(), BudgetObserver::new());
    let composed = measure(|| run(&mut shipped)).expect("counting allocator installed");
    assert_eq!(composed.calls, 0, "timing + budget: no remark consumer");

    // The control: a consumer attached, the same body allocates one label
    // per pass - and the counter itself allocates nothing.
    let mut counter = RemarkCounter::new();
    let attached = measure(|| run(&mut counter)).expect("counting allocator installed");
    assert_eq!(attached.calls, 3, "one label per pass");
    assert_eq!(counter.analysis, 3);
}
