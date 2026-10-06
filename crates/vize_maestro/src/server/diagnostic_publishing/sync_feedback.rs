//! Prompt edit feedback before waiting for native collection ownership.

use std::{future::Future, task::Poll};
use tower_lsp::lsp_types::{Diagnostic, Url};

use super::super::{MaestroServer, state::CorsaRequestStamp};
use crate::ide::DiagnosticService;

pub(in crate::server) struct SyncDiagnostics {
    pub(super) version: i32,
    pub(super) stamp: CorsaRequestStamp,
    pub(super) diagnostics: Vec<Diagnostic>,
}

impl MaestroServer {
    /// Reuse the initial notification contract: unversioned means partial.
    /// Edits also publish empty sets to clear a repaired lint/parser warning.
    /// This pass does not own or retire the pending complete diagnostic job.
    pub(in crate::server) async fn publish_changed_sync_diagnostics(
        &self,
        uri: &Url,
        version: i32,
    ) -> Option<SyncDiagnostics> {
        if !self.state.is_lsp_typecheck_enabled()
            || !self.state.lsp_features().has_diagnostics()
            || self.state.documents.version(uri) != Some(version)
        {
            return None;
        }
        let stamp = self.state.corsa_request_stamp();
        let diagnostics = DiagnosticService::collect(&self.state, uri);
        if !stamp.is_current(&self.state) {
            self.retry_current_diagnostics(uri);
            return None;
        }
        self.state
            .cache_lint_hover_diagnostics(uri, version, &diagnostics);
        let notification = self
            .client
            .publish_diagnostics(uri.clone(), diagnostics.clone(), None);
        futures::pin_mut!(notification);
        let document = self.state.diagnostic_lock(uri);
        let mut queued = false;
        futures::future::poll_fn(|context| {
            if queued {
                return notification.as_mut().poll(context);
            }
            match document.enqueue_sync(&self.state, uri, version, stamp, || {
                notification.as_mut().poll(context)
            }) {
                Some(result) => {
                    // The pinned tower-lsp client queues on its first poll,
                    // before any Pending from flushing its cloned sender.
                    queued = true;
                    result
                }
                None => Poll::Ready(()),
            }
        })
        .await;
        Some(SyncDiagnostics {
            version,
            stamp,
            diagnostics,
        })
    }
}
