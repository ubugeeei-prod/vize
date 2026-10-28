//! Diagnostic collection and LSP transport publishing.

#[cfg(feature = "native")]
use std::{future::Future, task::Poll};
#[cfg(feature = "native")]
use tower_lsp::lsp_types::MessageType;
use tower_lsp::lsp_types::{Diagnostic, Url};

use crate::ide::DiagnosticService;

use super::MaestroServer;

mod complete;
#[cfg(all(test, feature = "native", unix))]
mod configuration_failure_tests;
#[cfg(all(test, feature = "native", unix))]
mod retry_tests;
#[cfg(feature = "native")]
mod sync_feedback;
#[cfg(feature = "native")]
pub(in crate::server) use sync_feedback::SyncDiagnostics;

pub(super) struct CollectedDiagnostics {
    pub(super) version: i32,
    pub(super) diagnostics: Vec<Diagnostic>,
    #[cfg(feature = "native")]
    pub(super) stamp: Option<super::state::CorsaRequestStamp>,
}

impl MaestroServer {
    /// Publish non-empty diagnostics that do not need Corsa, provided the
    /// document still has the version opened by the caller. Empty initial
    /// results are withheld so consumers do not mistake the parser/lint pass
    /// for the terminal combined result from the queued type-diagnostic pass.
    /// Non-empty sync results are still useful as prompt feedback, but they
    /// stay unversioned for the same reason: a versioned publish is the
    /// complete parser/lint/typecheck answer for that document version.
    ///
    /// This deliberately bypasses `publish_collected_diagnostics`: Corsa has
    /// not been attempted yet, so its one-shot "type checking unavailable"
    /// notice would be premature here.
    #[cfg(feature = "native")]
    pub(super) async fn publish_initial_sync_diagnostics(&self, uri: &Url, expected: i32) {
        let diagnostic_lock = self.state.diagnostic_lock(uri);
        let diagnostic_guard = diagnostic_lock.lock().await;
        let stamp = self.state.corsa_request_stamp();

        let diagnostics = if self.state.documents.version(uri) == Some(expected) {
            if self.state.lsp_features().has_diagnostics() {
                Some(DiagnosticService::collect(&self.state, uri))
            } else {
                Some(Vec::new())
            }
        } else {
            None
        };

        drop(diagnostic_guard);

        if let Some(diagnostics) = diagnostics
            && self.state.documents.version(uri) == Some(expected)
            && stamp.is_current(&self.state)
        {
            self.state
                .cache_lint_hover_diagnostics(uri, expected, &diagnostics);
            if !diagnostics.is_empty() {
                self.client
                    .publish_diagnostics(uri.clone(), diagnostics, None)
                    .await;
            }
        }
    }

    /// Collect diagnostics while the caller owns this document's diagnostic
    /// lock. Sending the LSP notification is deliberately separate: the
    /// client channel can apply backpressure, and no document lock should be
    /// held while waiting for the transport to drain.
    pub(super) async fn collect_diagnostics_unlocked(
        &self,
        uri: &Url,
        expected: Option<i32>,
    ) -> Option<CollectedDiagnostics> {
        self.collect_diagnostics_with_sync_unlocked(
            uri,
            expected,
            #[cfg(feature = "native")]
            None,
        )
        .await
    }

    pub(super) async fn collect_diagnostics_with_sync_unlocked(
        &self,
        uri: &Url,
        expected: Option<i32>,
        #[cfg(feature = "native")] prepared: Option<SyncDiagnostics>,
    ) -> Option<CollectedDiagnostics> {
        #[cfg(feature = "native")]
        if self.state.project_context_retired() {
            return None;
        }
        #[cfg(feature = "native")]
        let scope = if self.state.is_lsp_typecheck_enabled() {
            Some(self.state.corsa_request_scope().await)
        } else {
            None
        };
        let version = self
            .state
            .documents
            .get(uri)
            .map(|document| document.version);

        // The native scope can wait behind an older request. Bind the pass to
        // the version that scheduled it only after that wait, so an old edit
        // or initial job cannot adopt and republish a newer editor revision.
        if expected.is_some() && version != expected {
            #[cfg(feature = "native")]
            self.retry_current_diagnostics(uri);
            return None;
        }

        if !self.state.lsp_features().has_diagnostics() {
            return version.map(|version| CollectedDiagnostics {
                version,
                diagnostics: Vec::new(),
                #[cfg(feature = "native")]
                stamp: None,
            });
        }

        let Some(version) = version else {
            tracing::debug!("skipping diagnostics for unopened document: {}", uri);
            return None;
        };

        // Use async version when native feature is enabled (includes Corsa diagnostics)
        #[cfg(feature = "native")]
        let diagnostics = {
            let sync = prepared
                .filter(|sync| sync.version == version && sync.stamp.is_current(&self.state))
                .map_or_else(
                    || DiagnosticService::collect(&self.state, uri),
                    |sync| sync.diagnostics,
                );
            let stamp = self.state.corsa_request_stamp();
            crate::ide::diagnostics::with_native_diagnostic_progress(
                self.state.native_diagnostic_timeout(),
                DiagnosticService::collect_async_from_sync(&self.state, uri, sync.clone()),
                || self.publish_pending_native_diagnostics(uri, version, stamp, &sync),
            )
            .await
        };

        #[cfg(not(feature = "native"))]
        let diagnostics = DiagnosticService::collect(&self.state, uri);

        #[cfg(feature = "native")]
        if scope.as_ref().is_some_and(|scope| !scope.is_current()) {
            self.retry_current_diagnostics(uri);
            return None;
        }

        let current_version = self
            .state
            .documents
            .get(uri)
            .map(|document| document.version);
        if current_version != Some(version) {
            #[cfg(feature = "native")]
            self.retry_current_diagnostics(uri);
            tracing::debug!(
                "skipping stale diagnostics for {}: collected version {}, current {:?}",
                uri,
                version,
                current_version
            );
            return None;
        }

        Some(CollectedDiagnostics {
            version,
            diagnostics,
            #[cfg(feature = "native")]
            stamp: scope.as_ref().map(|scope| scope.stamp()),
        })
    }

    /// Pending feedback is partial and must not claim a terminal publication or
    /// complete the scheduled job. Reuse the captured source/config stamp.
    #[cfg(feature = "native")]
    pub(in crate::server) async fn publish_pending_native_diagnostics(
        &self,
        uri: &Url,
        version: i32,
        stamp: super::state::CorsaRequestStamp,
        sync: &[Diagnostic],
    ) {
        if self.state.project_context_retired()
            || !self.state.is_lsp_typecheck_enabled()
            || !stamp.is_current(&self.state)
            || self.state.documents.version(uri) != Some(version)
        {
            return;
        }
        let mut diagnostics = sync.to_vec();
        diagnostics.push(crate::ide::diagnostics::typecheck_pending_hint());
        let notification = self
            .client
            .publish_diagnostics(uri.clone(), diagnostics, None);
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
                    queued = true;
                    result
                }
                None => Poll::Ready(()),
            }
        })
        .await;
    }

    #[cfg(feature = "native")]
    async fn publish_typecheck_unavailable_notice(&self) {
        // Surface a one-shot UI notification when type checking is requested
        // but Corsa never came up. The hint diagnostic emitted by
        // collect_async (see #708) shows up in the Problems panel; this
        // adds a window/showMessage so users with the Problems panel
        // collapsed also notice. See #681.
        if self.state.is_lsp_typecheck_enabled()
            && !self.state.has_corsa_bridge()
            && self.state.claim_typecheck_unavailable_notice()
        {
            self.client
                .show_message(
                    MessageType::WARNING,
                    crate::ide::diagnostics::TYPECHECK_UNAVAILABLE_NOTICE_MESSAGE,
                )
                .await;
        }
    }
}
