//! Editor text mutation and diagnostic refresh.

use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use super::MaestroServer;

impl MaestroServer {
    /// Publish the changed document first, then refresh open typed files that
    /// directly import it. Corsa has already received the changed virtual
    /// document by this point, so importer diagnostics observe the new shape.
    pub(crate) async fn apply_document_changes(
        &self,
        uri: &Url,
        changes: Vec<TextDocumentContentChangeEvent>,
        version: i32,
    ) {
        if !self.state.documents.apply_changes(uri, changes, version) {
            return;
        }
        let project = self.for_document(uri).await;
        project.publish_changed_document(uri, version).await;
    }

    async fn publish_changed_document(&self, uri: &Url, version: i32) {
        if self.state.documents.version(uri) != Some(version) {
            return;
        }
        #[cfg(feature = "experimental-source-navigation")]
        self.notify_native_navigation(uri);

        let Some(content) = self.state.documents.text(uri) else {
            return;
        };
        self.state.update_virtual_docs(uri, &content);
        #[cfg(feature = "native")]
        let retained = super::initial_diagnostics::RetainedDiagnostics::new(self, uri);
        #[cfg(feature = "native")]
        super::workspace_files::invalidate_changed_document_disk_project_state(self, uri).await;
        #[cfg(feature = "native")]
        let sync = self.publish_changed_sync_diagnostics(uri, version).await;
        // Apply editor text before waiting for type diagnostics. Their lock
        // cannot delay didChange or hide the new revision from pending replies.
        #[cfg(feature = "native")]
        let diagnostic_lock = self.state.diagnostic_lock(uri);
        #[cfg(feature = "native")]
        let diagnostic_guard = diagnostic_lock.lock().await;

        let diagnostics = self
            .collect_diagnostics_with_sync_unlocked(
                uri,
                Some(version),
                #[cfg(feature = "native")]
                sync,
            )
            .await;

        #[cfg(feature = "native")]
        drop(diagnostic_guard);

        if let Some(diagnostics) = diagnostics {
            self.publish_collected_diagnostics(uri, diagnostics).await;
        }
        #[cfg(feature = "native")]
        retained.finish();

        tracing::info!(
            "starting importer diagnostics after edit {} version {}",
            uri,
            version
        );
        self.publish_importer_diagnostics(uri, Some(version)).await;
        tracing::info!(
            "finished importer diagnostics after edit {} version {}",
            uri,
            version
        );
    }
}
