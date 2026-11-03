//! Fault hooks are private thread-local test state, absent from production.
extern crate std;
use std::cell::Cell;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Fault {
    BeforeObserve,
    AfterPark,
    AfterMint,
    AfterAttach,
}
std::thread_local! {
    static FAULT: Cell<Option<Fault>> = const { Cell::new(None) };
    static OBSERVATIONS: Cell<usize> = const { Cell::new(0) };
}
pub(super) fn set_fault(fault: Fault) {
    FAULT.with(|state| state.set(Some(fault)));
}
pub(super) fn reset_observations() {
    OBSERVATIONS.with(|state| state.set(0));
}
pub(super) fn observations() -> usize {
    OBSERVATIONS.with(Cell::get)
}
fn trip(fault: Fault) {
    FAULT.with(|state| {
        if state.get() == Some(fault) {
            state.set(None);
            panic!("private fault at original prepared root Text event");
        }
    });
}
pub(super) fn before_observe() {
    trip(Fault::BeforeObserve);
    OBSERVATIONS.with(|state| state.set(state.get() + 1));
}
pub(super) fn after_park() {
    trip(Fault::AfterPark);
}
pub(super) fn after_mint() {
    trip(Fault::AfterMint);
}
pub(super) fn after_attach() {
    trip(Fault::AfterAttach);
}
