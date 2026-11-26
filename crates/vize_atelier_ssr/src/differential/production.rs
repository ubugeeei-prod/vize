//! Production-path lane control for the SSR corpus gate.
//!
//! [`super::compare_ssr_lanes`] compiles one template under options the caller
//! derives. That can drift from what adapters actually pass (the SFC compiler
//! adds a Croquis summary for `<script setup>`, for example), so the corpus
//! gate also drives the real SFC entry point. These thread-local switches let
//! it run that entry point twice, once per lane, and read each production SSR
//! compile's lane verdict. They exist only under `legacy-differential`; the
//! published compile path always selects.

use core::cell::{Cell, RefCell};

use crate::compile::SsrLane;

std::thread_local! {
    static PINNED_LEGACY: Cell<bool> = const { Cell::new(false) };
    static VERDICTS: RefCell<Option<std::vec::Vec<&'static str>>> = const { RefCell::new(None) };
}

/// Run `f` with every production SSR compile on this thread pinned to the
/// legacy AST walker.
pub fn with_legacy_lane<R>(f: impl FnOnce() -> R) -> R {
    let previous = PINNED_LEGACY.with(|pinned| pinned.replace(true));
    let result = f();
    PINNED_LEGACY.with(|pinned| pinned.set(previous));
    result
}

/// Run `f` and return, beside its result, the lane verdict of every
/// production SSR compile it made on this thread, in compile order.
pub fn record_lanes<R>(f: impl FnOnce() -> R) -> (R, std::vec::Vec<&'static str>) {
    let previous = VERDICTS.with(|verdicts| verdicts.replace(Some(std::vec::Vec::new())));
    let result = f();
    let recorded = VERDICTS.with(|verdicts| verdicts.replace(previous));
    (result, recorded.unwrap_or_default())
}

/// Run one production compile on the selected lane, recording its lane
/// verdicts, and again with the legacy walker pinned.
pub fn on_both_lanes<R>(compile: impl Fn() -> R) -> (R, std::vec::Vec<&'static str>, R) {
    let (selected, verdicts) = record_lanes(&compile);
    let legacy = with_legacy_lane(compile);
    (selected, verdicts, legacy)
}

/// The lane a production compile takes on this thread.
pub(crate) fn production_lane() -> SsrLane {
    if PINNED_LEGACY.with(Cell::get) {
        SsrLane::LegacyOnly
    } else {
        SsrLane::Selected
    }
}

/// Record a selected-lane verdict, its [`crate::l4::lane_label`], while
/// [`record_lanes`] is active.
pub(crate) fn record_verdict(label: &'static str) {
    VERDICTS.with(|verdicts| {
        if let Some(verdicts) = verdicts.borrow_mut().as_mut() {
            verdicts.push(label);
        }
    });
}

/// Run `f` without recording lane verdicts: the direct lane's debug oracle
/// recompiles on the full path, and that compile is not a production one.
#[cfg(debug_assertions)]
pub(crate) fn unrecorded<R>(f: impl FnOnce() -> R) -> R {
    let previous = VERDICTS.with(|verdicts| verdicts.replace(None));
    let result = f();
    VERDICTS.with(|verdicts| verdicts.replace(previous));
    result
}

/// Record the verdict of a compile the direct lane emitted.
pub(crate) fn record_emitted() {
    VERDICTS.with(|verdicts| {
        if let Some(verdicts) = verdicts.borrow_mut().as_mut() {
            verdicts.push("s4");
        }
    });
}
