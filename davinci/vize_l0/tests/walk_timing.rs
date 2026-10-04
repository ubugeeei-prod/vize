//! Caller marks preserve real fused-walk lifecycle without a clock or sink.

#![expect(
    clippy::expect_used,
    reason = "laws require their actual pipeline events"
)]

use core::cell::Cell;

use vize_l0::pass::{
    FailEvent, Fusability, PassDesc, PassEvent, PassFailure, PassKind, PassObserver, Pipeline,
    Preserved, WalkTiming, run_pipeline,
};

const PLAN: Pipeline = Pipeline::new(
    "s2",
    &[
        PassDesc::new(
            "normalize",
            PassKind::Optional,
            Fusability::Fusable,
            Preserved::ALL,
        ),
        PassDesc::new(
            "fold",
            PassKind::Optional,
            Fusability::Fusable,
            Preserved::ALL,
        ),
        PassDesc::new(
            "verify",
            PassKind::MandatoryDiagnostic,
            Fusability::Barrier,
            Preserved::ALL,
        ),
    ],
);

// Deliberately no Copy, Clone or Default: the returned mark belongs to its caller.
#[derive(Debug)]
struct Mark<'a> {
    id: u32,
    drops: &'a Cell<u32>,
}

impl Drop for Mark<'_> {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

struct Observer<'a> {
    timing: WalkTiming<Mark<'a>>,
    starts: &'a Cell<u32>,
    drops: &'a Cell<u32>,
    enabled: bool,
    completed: [Option<(u32, &'static str, &'static str)>; 2],
    samples: usize,
}

impl<'a> Observer<'a> {
    fn new(starts: &'a Cell<u32>, drops: &'a Cell<u32>, enabled: bool) -> Self {
        Self {
            timing: WalkTiming::default(),
            starts,
            drops,
            enabled,
            completed: [None; 2],
            samples: 0,
        }
    }
}

impl PassObserver for Observer<'_> {
    fn before_pipeline(&mut self, _pipeline: &Pipeline) {
        self.timing.discard();
    }

    fn before_pass(&mut self, event: &PassEvent<'_>) {
        let (starts, drops, enabled) = (self.starts, self.drops, self.enabled);
        self.timing.begin(event, || {
            starts.set(starts.get() + 1);
            enabled.then(|| Mark {
                id: starts.get(),
                drops,
            })
        });
    }

    fn after_pass(&mut self, event: &PassEvent<'_>) {
        if let Some((mark, attribution)) = self.timing.end(event) {
            *self
                .completed
                .get_mut(self.samples)
                .expect("two real walks") = Some((
                mark.id,
                attribution.stage.expect("actual stage"),
                attribution.pass.expect("actual lead"),
            ));
            self.samples += 1;
            drop(mark);
        }
    }

    fn on_fail(&mut self, _event: &FailEvent<'_>) {
        self.timing.discard();
    }
}

fn event(index: usize) -> PassEvent<'static> {
    let group_index = PLAN.group_of_pass(index).expect("actual registered pass");
    PassEvent {
        pipeline: &PLAN,
        group_index,
        group: PLAN.group(group_index).expect("actual fusion group"),
        pass_index: index,
    }
}

#[test]
fn non_copy_marks_follow_actual_fused_and_barrier_callbacks() {
    let (starts, drops) = (Cell::new(0), Cell::new(0));
    let mut observer = Observer::new(&starts, &drops, true);
    run_pipeline(&PLAN, &mut observer, |_event| Ok(())).expect("real three-pass runner");
    assert_eq!(
        starts.get(),
        2,
        "lazy factory runs once per walk, never per pass"
    );
    assert_eq!(observer.samples, 2);
    assert_eq!(
        observer.completed,
        [Some((1, "s2", "normalize")), Some((2, "s2", "verify"))]
    );
    assert_eq!(
        drops.get(),
        2,
        "only each actual recipient drops its owned mark"
    );
    drop(observer);
    assert_eq!(
        drops.get(),
        2,
        "completed marks are not retained by the observer"
    );
}

#[test]
fn intermediate_callbacks_never_start_or_consume_a_mark() {
    let drops = Cell::new(0);
    let mut timing = WalkTiming::new();
    timing.begin(&event(1), || {
        panic!("non-entry callback invoked the factory")
    });
    assert!(
        timing.end(&event(1)).is_none(),
        "no fabricated completed walk"
    );
    timing.begin(&event(0), || {
        Some(Mark {
            id: 7,
            drops: &drops,
        })
    });
    assert!(
        timing.end(&event(0)).is_none(),
        "fused entry is not its exit"
    );
    timing.begin(&event(1), || {
        panic!("second fused pass invoked the factory")
    });
    let (mark, attribution) = timing.end(&event(1)).expect("actual group exit");
    assert_eq!(mark.id, 7);
    assert_eq!(attribution.stage, Some("s2"));
    assert_eq!(attribution.pass, Some("normalize"));
    assert_eq!(
        drops.get(),
        0,
        "mark transfers without being cloned or dropped"
    );
    assert!(
        timing.end(&event(1)).is_none(),
        "exit cannot return a mark twice"
    );
    drop(mark);
    assert_eq!(drops.get(), 1);
}

#[test]
fn declined_starts_produce_no_mark_or_completed_sample() {
    let (starts, drops) = (Cell::new(0), Cell::new(0));
    let mut observer = Observer::new(&starts, &drops, false);
    run_pipeline(&PLAN, &mut observer, |_event| Ok(())).expect("disabled caller still runs");
    assert_eq!(starts.get(), 2, "enable check remains once per true entry");
    assert_eq!(observer.samples, 0);
    assert_eq!(observer.completed, [None; 2]);
    assert_eq!(drops.get(), 0);
}

#[test]
fn actual_runner_failure_discards_an_owned_mark_and_can_restart() {
    let (starts, drops) = (Cell::new(0), Cell::new(0));
    let mut observer = Observer::new(&starts, &drops, true);
    let failed = run_pipeline(&PLAN, &mut observer, |_event| {
        Err(PassFailure::new("original failure"))
    });
    assert_eq!(failed, Err(PassFailure::new("original failure")));
    assert_eq!(starts.get(), 1);
    assert_eq!(drops.get(), 1, "on_fail discards the interrupted walk");
    assert_eq!(
        observer.samples, 0,
        "failed work is not a successful timing sample"
    );
    run_pipeline(&PLAN, &mut observer, |_event| Ok(())).expect("same observer restarts");
    assert_eq!(
        observer.completed,
        [Some((2, "s2", "normalize")), Some((3, "s2", "verify"))]
    );
    assert_eq!(drops.get(), 3);
}

#[test]
fn pipeline_restart_discards_the_previous_open_mark() {
    let (starts, drops) = (Cell::new(0), Cell::new(0));
    let mut observer = Observer::new(&starts, &drops, true);
    observer.before_pipeline(&PLAN);
    observer.before_pass(&event(0));
    assert_eq!(starts.get(), 1);
    assert_eq!(drops.get(), 0);
    observer.before_pipeline(&PLAN);
    assert_eq!(drops.get(), 1, "abandoned walk cannot span runs");
    observer.after_pass(&event(1));
    assert_eq!(observer.samples, 0);
    run_pipeline(&PLAN, &mut observer, |_event| Ok(())).expect("fresh callbacks complete");
    assert_eq!(
        observer.completed,
        [Some((2, "s2", "normalize")), Some((3, "s2", "verify"))]
    );
    assert_eq!(drops.get(), 3);
}

#[test]
fn dropping_an_open_state_drops_its_mark_exactly_once() {
    let drops = Cell::new(0);
    let mut timing = WalkTiming::new();
    timing.begin(&event(0), || {
        Some(Mark {
            id: 9,
            drops: &drops,
        })
    });
    assert_eq!(drops.get(), 0);
    drop(timing);
    assert_eq!(drops.get(), 1);
}
