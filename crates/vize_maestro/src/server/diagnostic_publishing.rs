//! Diagnostic collection and LSP transport publishing.

#[cfg(feature = "native")]
use tower_lsp::lsp_types::MessageType;
use tower_lsp::lsp_types::{Diagnostic, Url};

use crate::ide::DiagnosticService;

use super::MaestroServer;

impl MaestroServer {
    /// Publish non-empty diagnostics that do not need Corsa, provided the
    /// document still has the version opened by the caller. Empty initial
    /// results are withheld so consumers do not mistake the parser/lint pass
    /// for the terminal combined result from the queued type-diagnostic pass.
    ///
    /// This deliberately bypasses `publish_collected_diagnostics`: Corsa has
    /// not been attempted yet, so its one-shot "type checking unavailable"
    /// notice would be premature here.
    #[cfg(feature = "native")]
    pub(super) async fn publish_initial_sync_diagnostics(&self, uri: &Url, expected: i32) {
        let diagnostic_lock = self.state.diagnostic_lock(uri);
        let diagnostic_guard = diagnostic_lock.lock().await;

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

        if let Some(diagnostics) = diagnostics.filter(|diagnostics| !diagnostics.is_empty())
            && self.state.documents.version(uri) == Some(expected)
        {
            self.client
                .publish_diagnostics(uri.clone(), diagnostics, Some(expected))
                .await;
        }
    }

    /// Collect diagnostics while the caller owns this document's diagnostic
    /// lock. Sending the LSP notification is deliberately separate: the
    /// client channel can apply backpressure, and no document lock should be
    /// held while waiting for the transport to drain.
    pub(super) async fn collect_diagnostics_unlocked(
        &self,
        uri: &Url,
    ) -> Option<(i32, Vec<Diagnostic>)> {
        let version = self
            .state
            .documents
            .get(uri)
            .map(|document| document.version);

        if !self.state.lsp_features().has_diagnostics() {
            return version.map(|version| (version, Vec::new()));
        }

        let Some(version) = version else {
            tracing::debug!("skipping diagnostics for unopened document: {}", uri);
            return None;
        };

        // Use async version when native feature is enabled (includes Corsa diagnostics)
        #[cfg(feature = "native")]
        let diagnostics = DiagnosticService::collect_async(&self.state, uri).await;

        #[cfg(not(feature = "native"))]
        let diagnostics = DiagnosticService::collect(&self.state, uri);

        let current_version = self
            .state
            .documents
            .get(uri)
            .map(|document| document.version);
        if current_version != Some(version) {
            tracing::debug!(
                "skipping stale diagnostics for {}: collected version {}, current {:?}",
                uri,
                version,
                current_version
            );
            return None;
        }

        Some((version, diagnostics))
    }

    pub(super) async fn publish_collected_diagnostics(
        &self,
        uri: &Url,
        version: i32,
        diagnostics: Vec<Diagnostic>,
    ) {
        if self.state.documents.version(uri) != Some(version) {
            tracing::debug!(
                "skipping superseded diagnostics for {}: collected version {}, current {:?}",
                uri,
                version,
                self.state.documents.version(uri)
            );
            return;
        }

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, Some(version))
            .await;

        // Surface a one-shot UI notification when type checking is requested
        // but Corsa never came up. The hint diagnostic emitted by
        // collect_async (see #708) shows up in the Problems panel; this
        // adds a window/showMessage so users with the Problems panel
        // collapsed also notice. See #681.
        #[cfg(feature = "native")]
        if self.state.is_lsp_typecheck_enabled()
            && !self.state.has_corsa_bridge()
            && self.state.claim_typecheck_unavailable_notice()
        {
            self.client
                .show_message(
                    MessageType::WARNING,
                    "Vize: type checking is unavailable in this workspace. \
                     Make sure tsconfig.json exists and the Corsa runtime is reachable.",
                )
                .await;
        }
    }
}
