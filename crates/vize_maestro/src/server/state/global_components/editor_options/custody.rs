//! Passive test custody after existing native synchronization succeeds.
#![cfg(test)]

use std::{cell::RefCell, path::PathBuf};

use serde::Serialize;
use tower_lsp::lsp_types::Url;
use vize_canon::CorsaVueVirtualDocument;
use vize_l0::String;

use super::ServerState;

#[derive(Debug, Serialize)]
pub(crate) struct OpenedProject {
    feature: &'static str,
    authored_uri: Url,
    source: String,
    version: i32,
    revision: u64,
    request_uri: String,
    root: Option<PathBuf>,
}

struct ActiveObservation {
    owner: usize,
    uri: Url,
    records: Vec<OpenedProject>,
}

thread_local! {
    static OPENED: RefCell<Option<ActiveObservation>> = const { RefCell::new(None) };
}

pub(crate) struct Observation;

impl Drop for Observation {
    fn drop(&mut self) {
        OPENED.with(|opened| *opened.borrow_mut() = None);
    }
}

impl ServerState {
    pub(crate) fn observe_editor_project_opens(&self, uri: &Url) -> Observation {
        OPENED.with(|opened| {
            assert!(opened.borrow().is_none(), "nested project observation");
            *opened.borrow_mut() = Some(ActiveObservation {
                owner: std::ptr::from_ref(self).addr(),
                uri: uri.clone(),
                records: Vec::new(),
            });
        });
        Observation
    }

    pub(crate) fn take_editor_project_opens(&self) -> Vec<OpenedProject> {
        OPENED.with(|opened| {
            let mut opened = opened.borrow_mut();
            let active = opened.as_mut().expect("active observation");
            assert_eq!(active.owner, std::ptr::from_ref(self).addr());
            std::mem::take(&mut active.records)
        })
    }

    pub(crate) fn record_editor_project_open(
        &self,
        feature: &'static str,
        uri: &Url,
        source: &str,
        document: &CorsaVueVirtualDocument,
    ) {
        OPENED.with(|opened| {
            if let Some(active) = opened.borrow_mut().as_mut()
                && active.owner == std::ptr::from_ref(self).addr()
                && active.uri == *uri
            {
                let (version, revision, current_source) = self
                    .documents
                    .get(uri)
                    .map(|document| (document.version, document.revision(), document.text()))
                    .expect("observed authored document");
                assert_eq!(current_source, source, "observed source generation");
                active.records.push(OpenedProject {
                    feature,
                    authored_uri: uri.clone(),
                    source: source.into(),
                    version,
                    revision,
                    request_uri: document.request_uri.clone(),
                    root: document.session_project_root.clone(),
                });
            }
        });
    }
}
