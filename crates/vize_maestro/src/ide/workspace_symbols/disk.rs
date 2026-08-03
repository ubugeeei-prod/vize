//! Workspace-symbol contributions from closed files created mid-session.

use tower_lsp::lsp_types::SymbolInformation;

use super::WorkspaceSymbolsService;
use crate::server::ServerState;

pub(super) fn collect(state: &ServerState, query: &str, symbols: &mut Vec<SymbolInformation>) {
    for uri in state.workspace_vue_file_uris() {
        // Open buffers remain authoritative when their unsaved text differs
        // from the corresponding file on disk.
        if state.documents.contains(&uri) {
            continue;
        }
        let Ok(path) = uri.to_file_path() else {
            continue;
        };
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        WorkspaceSymbolsService::collect_symbols_from_document(&uri, &content, query, symbols);
    }
}
