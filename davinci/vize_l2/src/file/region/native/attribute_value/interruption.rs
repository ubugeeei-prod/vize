//! Test-only faults preserve the production source event and whole walk guard.
extern crate std;
use std::cell::Cell;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Fault {
    BeforeObserve,
    AfterPark,
    AfterClose,
    AfterAttach,
}
std::thread_local! { static FAULT: Cell<Option<Fault>> = const { Cell::new(None) }; }
pub(super) fn set_fault(fault: Fault) {
    FAULT.with(|state| state.set(Some(fault)));
}
pub(super) fn trip(fault: Fault) {
    FAULT.with(|state| {
        if state.get() == Some(fault) {
            state.set(None);
            panic!("test fault during original attribute value event");
        }
    });
}
