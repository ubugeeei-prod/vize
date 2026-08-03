//! Closed Vue files announced through workspace file-operation events.

use tower_lsp::lsp_types::Url;

use super::ServerState;

impl ServerState {
    /// Track a newly-created on-disk Vue file without treating it as an open
    /// editor document. Returns false for non-file, non-Vue, or missing URIs.
    pub(crate) fn track_workspace_vue_file(&self, uri: &str) -> bool {
        let Ok(uri) = Url::parse(uri) else {
            return false;
        };
        if !uri.path().ends_with(".vue") || !uri.to_file_path().is_ok_and(|path| path.is_file()) {
            return false;
        }
        self.workspace_vue_files.insert(uri, ()).is_none()
    }

    /// Forget a deleted or renamed on-disk Vue file.
    pub(crate) fn forget_workspace_vue_file(&self, uri: &str) -> bool {
        let Ok(uri) = Url::parse(uri) else {
            return false;
        };
        self.workspace_vue_files.remove(&uri).is_some()
    }

    /// Stable URI snapshot for workspace-wide searches. The caller performs
    /// filesystem reads after all DashMap guards have been released.
    pub(crate) fn workspace_vue_file_uris(&self) -> Vec<Url> {
        let mut uris = self
            .workspace_vue_files
            .iter()
            .map(|entry| entry.key().clone())
            .collect::<Vec<_>>();
        uris.sort_by(|left, right| left.as_str().cmp(right.as_str()));
        uris
    }
}
