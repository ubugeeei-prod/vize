//! Test-only custody from the actual successful category requests.

use serde_json::Value;
use std::cell::RefCell;

std::thread_local! {
    static OBSERVED: RefCell<Option<Value>> = const { RefCell::new(None) };
}

pub(super) fn reset() {
    OBSERVED.with(|value| *value.borrow_mut() = None);
}

pub(super) fn record(receipt: Value) {
    OBSERVED.with(|value| *value.borrow_mut() = Some(receipt));
}

pub(in crate::lsp_client::editor_lsp) fn take() -> Option<Value> {
    OBSERVED.with(|value| value.borrow_mut().take())
}
