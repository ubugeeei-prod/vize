//! Test-only live-item census; no allocation or counter operand survives normal builds.

use serde::Serialize;
use std::cell::RefCell;

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub(super) struct Census {
    pub(super) calls: u64,
    pub(super) visited_live_items: u64,
    pub(super) eligible_live_items: u64,
    pub(super) selected: u64,
    pub(super) len_max: usize,
    pub(super) capacity_max: usize,
    pub(super) len_last: usize,
    pub(super) capacity_last: usize,
}
thread_local! {
    static CENSUS: RefCell<[Census; 4]> = RefCell::new([Census::default(); 4]);
}
pub(super) fn record(
    selector: usize,
    visited: u64,
    eligible: u64,
    len: usize,
    capacity: usize,
    selected: bool,
) {
    CENSUS.with(|state| {
        let mut state = state.borrow_mut();
        let row = &mut state[selector];
        row.calls += 1;
        row.visited_live_items += visited;
        row.eligible_live_items += eligible;
        row.selected += u64::from(selected);
        row.len_max = row.len_max.max(len);
        row.capacity_max = row.capacity_max.max(capacity);
        row.len_last = len;
        row.capacity_last = capacity;
    });
}
pub(super) fn reset() {
    CENSUS.with(|state| *state.borrow_mut() = [Census::default(); 4]);
}
pub(super) fn snapshot() -> [Census; 4] {
    CENSUS.with(|state| *state.borrow())
}
