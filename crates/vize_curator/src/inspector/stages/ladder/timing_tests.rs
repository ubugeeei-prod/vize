//! Actual Curator callbacks discard failed/restarted marks without clock reads.

use core::cell::Cell;

use vize_l0::pass::{
    Fusability, PassDesc, PassEvent, PassFailure, PassKind, PassObserver, Pipeline, Preserved,
    run_pipeline,
};

use super::{PassStart, PassWindows, step};

const PLAN: Pipeline = Pipeline::new(
    "s2",
    &[
        PassDesc::new("a", PassKind::Optional, Fusability::Fusable, Preserved::ALL),
        PassDesc::new("b", PassKind::Optional, Fusability::Fusable, Preserved::ALL),
    ],
);

fn event(index: usize) -> PassEvent<'static> {
    PassEvent {
        pipeline: &PLAN,
        group_index: 0,
        group: PLAN.group(0).expect("actual registered fusion group"),
        pass_index: index,
    }
}

#[test]
fn failed_curator_walk_is_discarded_and_the_next_run_uses_only_its_samples() {
    let reads = Cell::new(0_u64);
    let clock = || {
        let now = reads.get() * 10;
        reads.set(reads.get() + 1);
        now
    };
    let windows = PassWindows::default();
    let mut observer = PassStart {
        clock: &clock,
        windows: &windows,
        pipeline: None,
    };
    let failure = PassFailure::new("original failure");
    assert_eq!(
        run_pipeline(&PLAN, &mut observer, |_event| Err(failure)),
        Err(failure)
    );
    assert_eq!(reads.get(), 1, "failure/discard performs no clock read");
    assert_eq!(
        windows.close(&event(1), 900),
        (step("s2", "b", 0, 900), None),
        "failed mark is not an observation"
    );
    let mut walk = None;
    run_pipeline(&PLAN, &mut observer, |event| {
        let (_, completed) = windows.close(event, clock());
        if completed.is_some() {
            walk = completed;
        }
        Ok(())
    })
    .expect("same real observer restarts");
    assert_eq!(
        reads.get(),
        5,
        "two restarted passes need only four existing samples"
    );
    assert_eq!(walk, Some(step("s2", "a", 10, 40)));
}

#[test]
fn interrupted_curator_pipeline_restart_discards_its_old_mark_without_a_sample() {
    let reads = Cell::new(0_u64);
    let clock = || {
        let now = reads.get() * 10;
        reads.set(reads.get() + 1);
        now
    };
    let windows = PassWindows::default();
    let mut observer = PassStart {
        clock: &clock,
        windows: &windows,
        pipeline: None,
    };
    observer.before_pipeline(&PLAN);
    observer.before_pass(&event(0));
    observer.before_pipeline(&PLAN);
    assert_eq!(
        reads.get(),
        1,
        "restart discards without consulting the host clock"
    );
    assert_eq!(
        windows.close(&event(1), 900),
        (step("s2", "b", 0, 900), None)
    );
    let mut walk = None;
    run_pipeline(&PLAN, &mut observer, |event| {
        let (_, completed) = windows.close(event, clock());
        if completed.is_some() {
            walk = completed;
        }
        Ok(())
    })
    .expect("fresh callbacks complete");
    assert_eq!(walk, Some(step("s2", "a", 10, 40)));
    assert_eq!(reads.get(), 5);
}
