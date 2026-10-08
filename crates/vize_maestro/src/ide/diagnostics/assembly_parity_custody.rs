//! Test-thread custody for the original CLI/editor assembly law.

use std::cell::RefCell;

use serde_json::{Value, json};
use tower_lsp::lsp_types::Url;

use crate::server::ServerState;

struct Active {
    owner: usize,
    uri: Url,
    generated_uri: Option<std::string::String>,
    rows: Vec<Value>,
}

thread_local! {
    static ACTIVE: RefCell<Option<Active>> = const { RefCell::new(None) };
}

pub(super) struct Observation;

impl Drop for Observation {
    fn drop(&mut self) {
        ACTIVE.with(|active| *active.borrow_mut() = None);
    }
}

pub(super) fn observe(state: &ServerState, uri: &Url) -> Observation {
    ACTIVE.with(|active| {
        assert!(active.borrow().is_none());
        *active.borrow_mut() = Some(Active {
            owner: std::ptr::from_ref(state).addr(),
            uri: uri.clone(),
            generated_uri: None,
            rows: Vec::new(),
        });
    });
    Observation
}

pub(super) fn record(state: &ServerState, uri: &Url, stage: &str, value: impl FnOnce() -> Value) {
    ACTIVE.with(|active| {
        if let Some(active) = active.borrow_mut().as_mut()
            && active.owner == std::ptr::from_ref(state).addr()
            && active.uri == *uri
        {
            active.rows.push(json!({"stage":stage,"value":value()}));
        }
    });
}

pub(super) fn generated(state: &ServerState, uri: &Url, generated_uri: &str) {
    ACTIVE.with(|active| {
        if let Some(active) = active.borrow_mut().as_mut()
            && active.owner == std::ptr::from_ref(state).addr()
            && active.uri == *uri
        {
            active.generated_uri = Some(generated_uri.into());
        }
    });
}

pub(super) fn error(
    state: &ServerState,
    uri: &Url,
    stage: &str,
    attempt: usize,
    error: &vize_canon::CorsaBridgeError,
) {
    record(
        state,
        uri,
        stage,
        || json!({"attempt":attempt,"error":vize_l0::cstr!("{error:?}")}),
    );
}

pub(super) fn native(generated_uri: &str, stage: &str, value: impl FnOnce() -> Value) {
    ACTIVE.with(|active| {
        if let Some(active) = active.borrow_mut().as_mut()
            && active.generated_uri.as_deref() == Some(generated_uri)
        {
            active
                .rows
                .push(json!({"stage":stage,"generated_uri":generated_uri,"value":value()}));
        }
    });
}

pub(super) fn take() -> Vec<Value> {
    ACTIVE.with(|active| {
        active
            .borrow_mut()
            .as_mut()
            .map_or_else(Vec::new, |active| std::mem::take(&mut active.rows))
    })
}
