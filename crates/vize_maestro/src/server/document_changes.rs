//! Editor text mutation and diagnostic refresh.

use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use super::MaestroServer;

impl MaestroServer {
    #[cfg(feature = "native")]
    pub(super) async fn publish_changed_diagnostics_if_version(
        &self,
        uri: &Url,
        version: i32,
        sync: Option<super::diagnostic_publishing::SyncDiagnostics>,
    ) {
        let retained = super::initial_diagnostics::RetainedDiagnostics::new(self, uri);
        let diagnostic_lock = self.state.diagnostic_lock(uri);
        let diagnostic_guard = diagnostic_lock.lock().await;
        let diagnostics = self
            .collect_diagnostics_with_sync_unlocked(uri, Some(version), sync)
            .await;
        drop(diagnostic_guard);

        if let Some(diagnostics) = diagnostics {
            self.publish_collected_diagnostics(uri, diagnostics).await;
        }
        retained.finish();
    }

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
        #[cfg(all(
            feature = "retained-program",
            not(feature = "experimental-source-navigation")
        ))]
        self.notify_retained_navigation(uri);

        let Some(content) = self.state.documents.text(uri) else {
            return;
        };
        self.state.update_virtual_docs(uri, &content);
        #[cfg(feature = "native")]
        super::workspace_files::invalidate_changed_document_disk_project_state(self, uri).await;
        #[cfg(feature = "native")]
        let sync = self.publish_changed_sync_diagnostics(uri, version).await;
        // Keep native validation off the notification/request lane. The
        // bounded worker retains only the newest version for this URI and
        // waits for a short typing quiet period before claiming Corsa. Hover,
        // completion and definition can therefore synchronize and answer from
        // the current buffer instead of sitting behind an obsolete project
        // diagnostic pass for every keystroke.
        #[cfg(feature = "native")]
        if self.state.is_lsp_typecheck_enabled()
            && self.schedule_changed_diagnostics(uri.clone(), version, sync.clone())
        {
            return;
        }
        #[cfg(feature = "native")]
        self.publish_changed_diagnostics_if_version(uri, version, sync)
            .await;
        #[cfg(not(feature = "native"))]
        if let Some(diagnostics) = self.collect_diagnostics_unlocked(uri, Some(version)).await {
            self.publish_collected_diagnostics(uri, diagnostics).await;
        }

        tracing::info!(
            "starting importer diagnostics after edit {} version {}",
            uri,
            version
        );
        #[cfg(feature = "native")]
        self.publish_importer_diagnostics_with_current_owner(uri, Some(version), true)
            .await;
        #[cfg(not(feature = "native"))]
        self.publish_importer_diagnostics(uri, Some(version)).await;
        tracing::info!(
            "finished importer diagnostics after edit {} version {}",
            uri,
            version
        );
    }
}
