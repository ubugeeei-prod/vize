//! LSP server implementation.
//!
//! This module contains the core LSP server using tower-lsp.

mod annotations;
mod auto_insert;
mod capabilities;
mod document_structure;
mod format;
mod handlers;
mod helpers;
mod importers;
mod state;
mod workspace_files;

pub use capabilities::server_capabilities;
#[cfg(feature = "native")]
pub use state::BatchTypeCheckCache;
pub use state::{LspFeatureConfig, ServerState};

use tower_lsp::{Client, ClientSocket, LspService};

use crate::document::DocumentStore;

/// The Maestro LSP server.
pub struct MaestroServer {
    /// LSP client for sending notifications
    client: Client,
    /// Server state
    state: ServerState,
}

impl MaestroServer {
    /// Create a new Maestro server instance.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            state: ServerState::new(),
        }
    }

    /// Get the document store.
    pub fn documents(&self) -> &DocumentStore {
        &self.state.documents
    }
}

/// Build the language service, including Volar's private automatic-insertion
/// request. `LanguageServer` cannot declare custom methods, so every transport
/// must use this builder rather than `LspService::new`.
pub(crate) fn build_lsp_service() -> (LspService<MaestroServer>, ClientSocket) {
    LspService::build(MaestroServer::new)
        .custom_method(auto_insert::AUTO_INSERT_METHOD, MaestroServer::auto_insert)
        .finish()
}
