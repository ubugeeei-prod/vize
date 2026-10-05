//! Only actual entry calls are counted, isolated to the armed test thread.
extern crate std;

use super::Vue1DescriptorObservation;
use core::cell::Cell;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct Counts {
    pub(super) splitters: usize,
    pub(super) components: usize,
    pub(super) parked: usize,
    pub(super) dropped: usize,
    pub(super) parked_diagnostics: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Fault {
    None,
    ComponentEntry,
    AfterPark,
}

#[derive(Clone, Copy)]
struct State {
    counts: Counts,
    fault: Fault,
}

std::thread_local! {
    static STATE: Cell<Option<State>> = const { Cell::new(None) };
}

pub(super) struct Armed;

impl Armed {
    pub(super) fn new(fault: Fault) -> Self {
        STATE.with(|state| {
            assert!(
                state.get().is_none(),
                "no nested measurement on this thread"
            );
            state.set(Some(State {
                counts: Counts::default(),
                fault,
            }));
        });
        Self
    }

    pub(super) fn counts(&self) -> Counts {
        STATE.with(|state| state.get().unwrap().counts)
    }
}

impl Drop for Armed {
    fn drop(&mut self) {
        STATE.with(|state| state.set(None));
    }
}

pub(crate) fn splitter_entry() {
    update(|state| state.counts.splitters += 1);
}

pub(crate) fn component_entry() {
    let mut panic = false;
    update(|state| {
        state.counts.components += 1;
        if state.fault == Fault::ComponentEntry {
            state.fault = Fault::None;
            panic = true;
        }
    });
    assert!(!panic, "injected fault at original Component entry");
}

pub(super) fn after_park(owner: &Vue1DescriptorObservation<'_>) {
    let mut panic = false;
    update(|state| {
        let component = owner.component().expect("normal original Component parked");
        assert!(core::ptr::eq(
            component.block().root_source(),
            owner.source()
        ));
        state.counts.parked += 1;
        state.counts.parked_diagnostics += component
            .bindings()
            .iter()
            .filter_map(|binding| binding.syntax())
            .map(|syntax| syntax.diagnostics().count())
            .sum::<usize>();
        if state.fault == Fault::AfterPark {
            state.fault = Fault::None;
            panic = true;
        }
    });
    assert!(
        !panic,
        "injected fault after normal original Component custody"
    );
}

impl Drop for Vue1DescriptorObservation<'_> {
    fn drop(&mut self) {
        update(|state| state.counts.dropped += 1);
    }
}

fn update(mut change: impl FnMut(&mut State)) {
    STATE.with(|cell| {
        if let Some(mut state) = cell.get() {
            change(&mut state);
            cell.set(Some(state));
        }
    });
}
