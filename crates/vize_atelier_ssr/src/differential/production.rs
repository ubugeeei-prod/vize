//! Production-path lane control for the SSR corpus gate.
//!
//! [`super::compare_ssr_lanes`] compiles one template under options the caller
//! derives. That can drift from what adapters actually pass (the SFC compiler
//! adds a Croquis summary for `<script setup>`, for example), so the corpus
//! gate also drives the real SFC entry point. These thread-local switches let
//! it run that entry point twice, once per lane, and read each production SSR
//! compile's lane verdict. They exist only under `davinci-differential`; the
//! published compile path always selects.

use core::cell::{Cell, RefCell};

use crate::compile::SsrLane;
use crate::s4::SsrS4Selection;

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

/// The lane a production compile takes on this thread.
pub(crate) fn production_lane() -> SsrLane {
    if PINNED_LEGACY.with(Cell::get) {
        SsrLane::LegacyOnly
    } else {
        SsrLane::Selected
    }
}

/// Record a selected-lane verdict while [`record_lanes`] is active.
pub(crate) fn record_verdict(selection: &SsrS4Selection) {
    VERDICTS.with(|verdicts| {
        if let Some(verdicts) = verdicts.borrow_mut().as_mut() {
            verdicts.push(lane_label(selection));
        }
    });
}

/// `s4` when the plan emitted, `legacy.<reason>` for a selected legacy
/// route, `rejected` for a broken plan invariant.
pub(crate) fn lane_label(selection: &SsrS4Selection) -> &'static str {
    match selection {
        SsrS4Selection::Emitted(_) => "s4",
        SsrS4Selection::Legacy(reason) => reason.counter_suffix(),
        SsrS4Selection::Rejected(_) => "rejected",
    }
}
