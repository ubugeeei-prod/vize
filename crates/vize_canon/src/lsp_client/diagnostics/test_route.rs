//! An opt-in test marker for the transport that returned this batch response.

use std::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::lsp_client) enum Route {
    Api,
    Editor,
    Unknown,
}

std::thread_local! {
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
    static ROUTE: Cell<Option<Route>> = const { Cell::new(None) };
}

pub(in crate::lsp_client) fn begin(enabled: bool) {
    ACTIVE.set(enabled);
    ROUTE.set(None);
}

pub(super) fn record(route: Route) {
    if ACTIVE.get() {
        ROUTE.set(Some(match ROUTE.get() {
            None => route,
            Some(previous) if previous == route => route,
            Some(_) => Route::Unknown,
        }));
    }
}

pub(in crate::lsp_client) fn take() -> Option<Route> {
    ACTIVE.set(false);
    ROUTE.take()
}

impl super::CorsaProjectClient {
    pub(crate) fn begin_batch_test_receipt(&self, enabled: bool) {
        begin(enabled);
    }
}

#[test]
fn receipt_routes_do_not_leak_between_requests_or_credit_mixed_responses() {
    begin(false);
    record(Route::Api);
    assert_eq!(take(), None);
    begin(true);
    record(Route::Api);
    assert_eq!(take(), Some(Route::Api));
    begin(true);
    record(Route::Editor);
    assert_eq!(take(), Some(Route::Editor));
    begin(true);
    record(Route::Api);
    record(Route::Editor);
    assert_eq!(take(), Some(Route::Unknown));
    begin(true);
    assert_eq!(take(), None);
}
