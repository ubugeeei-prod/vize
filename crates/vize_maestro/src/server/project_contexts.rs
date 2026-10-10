//! Borrow the primary server or dispatch a document through its isolated project.
#![cfg_attr(
    feature = "native",
    expect(
        clippy::disallowed_types,
        reason = "document dispatch retains its cached project owner"
    )
)]

use std::ops::Deref;
use tower_lsp::lsp_types::Url;

use super::MaestroServer;

#[cfg_attr(
    all(feature = "native", feature = "experimental-source-navigation"),
    expect(
        clippy::large_enum_variant,
        reason = "stack views avoid allocation on every document request"
    )
)]
pub(super) enum DocumentServer<'a> {
    Primary(&'a MaestroServer),
    #[cfg(feature = "native")]
    Project(MaestroServer),
}

impl Deref for DocumentServer<'_> {
    type Target = MaestroServer;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Primary(server) => server,
            #[cfg(feature = "native")]
            Self::Project(server) => server,
        }
    }
}

impl MaestroServer {
    #[cfg(feature = "native")]
    pub(super) fn with_project_state(&self, state: std::sync::Arc<super::ServerState>) -> Self {
        Self {
            client: self.client.clone(),
            #[cfg(feature = "experimental-source-navigation")]
            navigation: Some(
                crate::source_project::navigation::NativeNavigationProject::new(
                    crate::source_project::SourceQueryProject::new_server(state.clone()),
                ),
            ),
            state,
            initial_diagnostics: self.initial_diagnostics.clone(),
            #[cfg(feature = "experimental-source-navigation")]
            module_link_termination: None,
        }
    }

    pub(super) fn for_document(&self, uri: &Url) -> DocumentServer<'_> {
        #[cfg(feature = "native")]
        if let Some(state) = self.state.document_project_state(uri) {
            return DocumentServer::Project(self.with_project_state(state));
        }
        #[cfg(not(feature = "native"))]
        let _ = uri;
        DocumentServer::Primary(self)
    }
}
