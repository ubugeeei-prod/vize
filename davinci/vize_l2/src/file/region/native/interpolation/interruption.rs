//! Thread-local test faults never alter production File facts or storage scans.
extern crate std;
use std::cell::Cell;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Fault {
    AfterPark,
    AfterMint,
}

std::thread_local! {
    static FAULT: Cell<Option<Fault>> = const { Cell::new(None) };
}

pub(super) fn set_fault(fault: Fault) {
    FAULT.with(|state| state.set(Some(fault)));
}

fn trip(fault: Fault) {
    FAULT.with(|state| {
        if state.get() == Some(fault) {
            state.set(None);
            panic!("test fault during original interpolation event");
        }
    });
}

pub(super) fn after_park() {
    trip(Fault::AfterPark);
}
pub(super) fn after_mint() {
    trip(Fault::AfterMint);
}
