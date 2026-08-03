//! Runtime workspace-folder event handling.

use tower_lsp::lsp_types::WorkspaceFoldersChangeEvent;

use super::MaestroServer;

impl MaestroServer {
    pub(super) async fn reconfigure_workspace_folders(&self, event: &WorkspaceFoldersChangeEvent) {
        let affected = self.state.apply_workspace_folders_change(event);
        for uri in affected {
            let Some(version) = self.state.documents.version(&uri) else {
                continue;
            };
            self.publish_diagnostics_if_version(&uri, version).await;
        }
    }
}
