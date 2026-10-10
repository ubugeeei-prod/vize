//! Runtime workspace-folder event handling.

use tower_lsp::lsp_types::WorkspaceFoldersChangeEvent;

use super::MaestroServer;

impl MaestroServer {
    pub(super) async fn reconfigure_workspace_folders(&self, event: &WorkspaceFoldersChangeEvent) {
        let affected = {
            #[cfg(feature = "native")]
            let _change = self.state.project_routing_change();
            let affected = self.state.apply_workspace_folders_change(event);
            #[cfg(feature = "native")]
            self.state.refresh_project_routes();
            affected
        };
        for uri in affected {
            let Some(version) = self.state.documents.version(&uri) else {
                continue;
            };
            self.for_document(&uri)
                .await
                .publish_diagnostics_if_version(&uri, version)
                .await;
        }
    }
}
