//! Diagnostic collection and LSP transport publishing.

#[cfg(feature = "native")]
use std::future::Future;
#[cfg(feature = "native")]
use tower_lsp::lsp_types::MessageType;
use tower_lsp::lsp_types::{Diagnostic, Url};

use crate::ide::DiagnosticService;

use super::MaestroServer;

#[cfg(all(test, feature = "native", unix))]
mod retry_tests;
#[cfg(feature = "native")]
mod sync_feedback;
#[cfg(feature = "native")]
use sync_feedback::SyncDiagnostics;

pub(super) struct CollectedDiagnostics {
    version: i32,
    diagnostics: Vec<Diagnostic>,
    #[cfg(feature = "native")]
    stamp: Option<super::state::CorsaRequestStamp>,
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
        let diagnostics = match prepared
            .filter(|sync| sync.version == version && sync.stamp.is_current(&self.state))
        {
            Some(sync) => {
                DiagnosticService::collect_async_from_sync(&self.state, uri, sync.diagnostics).await
            }
            None => DiagnosticService::collect_async(&self.state, uri).await,
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

    pub(super) async fn publish_collected_diagnostics(
        &self,
        uri: &Url,
        collected: CollectedDiagnostics,
    ) {
        self.publish_collected_diagnostics_with_cause(uri, collected, false)
            .await;
    }

    pub(super) async fn publish_collected_diagnostics_with_cause(
        &self,
        uri: &Url,
        collected: CollectedDiagnostics,
        explicit_save: bool,
    ) {
        // Without native work there is no asynchronous stamped coalescing;
        // both causes retain the ordinary complete publication below.
        #[cfg(not(feature = "native"))]
        let _ = explicit_save;
        let CollectedDiagnostics {
            version,
            diagnostics,
            #[cfg(feature = "native")]
            stamp,
        } = collected;
        #[cfg(feature = "native")]
        if self.state.project_context_retired()
            || stamp.is_some_and(|stamp| !stamp.is_current(&self.state))
        {
            self.retry_current_diagnostics(uri);
            return;
        }
        tracing::info!(
            "publishing collected diagnostics for {} version {}",
            uri,
            version
        );
        if self.state.documents.version(uri) != Some(version) {
            #[cfg(feature = "native")]
            self.retry_current_diagnostics(uri);
            tracing::debug!(
                "skipping superseded diagnostics for {}: collected version {}, current {:?}",
                uri,
                version,
                self.state.documents.version(uri)
            );
            return;
        }
        // A taken initial/retry job may outlive a foreground publish. Claim
        // the complete result for this source/environment, not just the root
        // version: dependency/configuration changes must still republish it.
        #[cfg(feature = "native")]
        let retained = super::initial_diagnostics::RetainedDiagnostics::new(self, uri);
        #[cfg(feature = "native")]
        let mut publication = if let Some(stamp) = stamp {
            let document = self.state.diagnostic_lock(uri);
            let claim = document.claim(
                &self.state,
                uri,
                version,
                stamp,
                &diagnostics,
                explicit_save,
            );
            if !stamp.is_current(&self.state) || self.state.documents.version(uri) != Some(version)
            {
                self.retry_current_diagnostics(uri);
                return;
            }
            let publication = match claim {
                Ok(Some(publication)) => publication,
                Ok(None) => {
                    if let Some(scheduler) = &self.initial_diagnostics {
                        scheduler.complete(uri, version);
                    }
                    retained.finish();
                    self.publish_typecheck_unavailable_notice().await;
                    return;
                }
                Err(_) => {
                    self.retry_current_diagnostics(uri);
                    return;
                }
            };
            Some(publication)
        } else {
            None
        };
        self.state
            .cache_lint_hover_diagnostics(uri, version, &diagnostics);
        tracing::info!(
            "cached collected diagnostics for {} version {}",
            uri,
            version
        );

        // An importer refresh, save, or edit may satisfy a queued initial pass.
        // Retire that job only after collecting the complete current result;
        // parser-only initial feedback deliberately bypasses this method.
        #[cfg(feature = "native")]
        if let Some(scheduler) = &self.initial_diagnostics {
            scheduler.complete(uri, version);
        }
        tracing::info!(
            "sending collected diagnostics for {} version {}",
            uri,
            version
        );

        #[cfg(feature = "native")]
        {
            let notification =
                self.client
                    .publish_diagnostics(uri.clone(), diagnostics, Some(version));
            futures::pin_mut!(notification);
            futures::future::poll_fn(|cx| {
                let result = notification.as_mut().poll(cx);
                // tower-lsp 0.20 clones the futures-channel 0.3 sender for
                // each notification. Its first poll enqueues into that
                // sender's reserved slot; Pending can then mean only flush
                // backpressure. Dropping that flush cannot retract the
                // queued notification, so retain its claim from this poll.
                if let Some(publication) = publication.take() {
                    publication.finish();
                }
                result
            })
            .await;
            retained.finish();
        }
        #[cfg(not(feature = "native"))]
        self.client
            .publish_diagnostics(uri.clone(), diagnostics, Some(version))
            .await;
        tracing::info!("sent collected diagnostics for {} version {}", uri, version);

        #[cfg(feature = "native")]
        self.publish_typecheck_unavailable_notice().await;
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
