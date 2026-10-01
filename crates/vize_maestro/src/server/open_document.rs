//! Document lifecycle and initial diagnostic scheduling.

use tower_lsp::lsp_types::{DidCloseTextDocumentParams, DidOpenTextDocumentParams};

use super::MaestroServer;

impl MaestroServer {
    pub(super) async fn open_document(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;
        let language_id = params.text_document.language_id;

        self.state
            .documents
            .open(uri.clone(), content.clone(), version, language_id);
        self.state.update_virtual_docs(&uri, &content);

        // Keep real parser/lint feedback immediate, but do not make the first
        // completion wait behind Corsa startup and a full type-diagnostic
        // pass. An empty intermediate result is withheld; the versioned worker
        // publishes the terminal combined result after its interactive grace.
        #[cfg(feature = "native")]
        if self.state.is_lsp_typecheck_enabled() {
            self.publish_initial_sync_diagnostics(&uri, version).await;
            if self.schedule_initial_diagnostics(uri.clone(), version) {
                return;
            }
        }

        self.publish_diagnostics(&uri).await;
        if self.state.is_lsp_typecheck_enabled() {
            self.publish_importer_diagnostics(&uri, Some(version)).await;
        }
    }

    pub(super) async fn close_document(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.state.close_document(&uri);

        // Clean up virtual documents cache
        self.state.remove_virtual_docs(&uri);

        // Clear diagnostics
        self.client
            .publish_diagnostics(uri.clone(), vec![], None)
            .await;
        if self.state.is_lsp_typecheck_enabled() {
            // Closing discards the unsaved dependency and restores disk types.
            // A concurrent reopen supersedes this refresh through None.
            self.publish_importer_diagnostics(&uri, None).await;
        }
    }
}
