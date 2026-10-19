//! Borrow the primary server or dispatch a document through its isolated project.
#![cfg_attr(
    feature = "native",
    expect(
        clippy::disallowed_types,
        reason = "document dispatch retains its cached project owner"
    )
)]

use std::ops::Deref;
use tower_lsp::lsp_types::{TextDocumentPositionParams, Url};

use super::MaestroServer;

#[cfg(feature = "native")]
#[derive(Clone, Default)]
pub(super) struct ConfigurationAdmissionReceipt {
    roots: Vec<std::path::PathBuf>,
}

#[cfg(feature = "native")]
impl ConfigurationAdmissionReceipt {
    fn covers(&self, root: &std::path::Path) -> bool {
        self.roots.iter().any(|known| known == root)
    }
    fn extend(&mut self, roots: impl IntoIterator<Item = std::path::PathBuf>) {
        self.roots.extend(roots);
        self.roots.sort();
        self.roots.dedup();
    }
}

enum DocumentOwner<'a> {
    Primary(&'a MaestroServer),
    #[cfg(feature = "native")]
    Project(MaestroServer),
}

pub(super) struct DocumentServer<'a> {
    owner: DocumentOwner<'a>,
    #[cfg(feature = "native")]
    authored_revision: Option<u64>,
}

impl<'a> DocumentServer<'a> {
    pub(super) fn primary(server: &'a MaestroServer) -> Self {
        Self {
            owner: DocumentOwner::Primary(server),
            #[cfg(feature = "native")]
            authored_revision: server.state.documents.stable_revision(),
        }
    }

    #[cfg(feature = "native")]
    pub(super) fn authored_revision_is_current(&self) -> bool {
        self.authored_revision.is_some()
            && self.authored_revision == self.state.documents.stable_revision()
    }
}

impl Deref for DocumentServer<'_> {
    type Target = MaestroServer;

    fn deref(&self) -> &Self::Target {
        match &self.owner {
            DocumentOwner::Primary(server) => server,
            #[cfg(feature = "native")]
            DocumentOwner::Project(server) => server,
        }
    }
}

impl MaestroServer {
    pub(super) async fn for_pos(
        &self,
        position: &TextDocumentPositionParams,
    ) -> DocumentServer<'_> {
        self.for_document(&position.text_document.uri).await
    }
    #[cfg(feature = "native")]
    pub(super) fn with_project_state(&self, state: std::sync::Arc<super::ServerState>) -> Self {
        Self {
            client: self.client.clone(),
            #[cfg(feature = "experimental-source-navigation")]
            navigation: if std::sync::Arc::ptr_eq(&state, &self.state) {
                self.navigation.clone()
            } else {
                state.cached_project_navigation()
            },
            state,
            initial_diagnostics: self.initial_diagnostics.clone(),
            configuration_watchers: self.configuration_watchers.clone(),
            configuration_admission: self.configuration_admission.clone(),
            #[cfg(feature = "experimental-source-navigation")]
            module_link_termination: None,
        }
    }

    pub(super) async fn for_document(&self, uri: &Url) -> DocumentServer<'_> {
        #[cfg(feature = "native")]
        {
            // A public dispatch always begins a new operation, even when its
            // caller retained an old document view from a previous operation.
            let mut receipt = ConfigurationAdmissionReceipt::default();
            self.for_document_in_operation(uri, &mut receipt).await
        }
        #[cfg(not(feature = "native"))]
        {
            let _ = uri;
            DocumentServer::primary(self)
        }
    }

    /// The first sync and type jobs are continuations of one didOpen, not new
    /// client requests. Their original full owner/input receipt must still pass
    /// fresh physical validation on the bounded configuration worker.
    #[cfg(feature = "native")]
    pub(super) async fn for_initial_document(
        &self,
        uri: &Url,
        version: i32,
        continuation: Option<&super::state::InitialConfigurationContinuation>,
    ) -> DocumentServer<'_> {
        let authored_revision = self.state.documents.stable_revision();
        if let Some(continuation) = continuation
            && let Some(state) = self
                .state
                .continue_initial_configuration(continuation.clone(), uri.clone(), version)
                .await
        {
            let server = self
                .with_project_state(state)
                .with_configuration_admission(&[continuation.root().to_path_buf()]);
            return DocumentServer {
                owner: DocumentOwner::Project(server),
                authored_revision,
            };
        }
        self.for_document(uri).await
    }

    #[cfg(feature = "native")]
    pub(super) async fn for_document_in_operation(
        &self,
        uri: &Url,
        receipt: &mut ConfigurationAdmissionReceipt,
    ) -> DocumentServer<'_> {
        let authored_revision = self.state.documents.stable_revision();
        if self
            .state
            .document_configuration_root(uri)
            .is_none_or(|root| !receipt.covers(&root))
        {
            let changes = self.state.refresh_document_configuration_async(uri).await;
            receipt.extend(changes.checked_roots);
            for error in changes.errors {
                self.client.show_message(error.typ, error.message).await;
            }
        }
        let mut project = self.for_document_admitted(uri).await;
        if let DocumentOwner::Project(server) = &mut project.owner {
            server.configuration_admission = receipt.clone();
        }
        // Admission can await configuration work; positions still belong to
        // the authored revision at the beginning of this operation.
        project.authored_revision = authored_revision;
        project
    }

    #[cfg(feature = "native")]
    pub(super) fn with_configuration_admission(&self, roots: &[std::path::PathBuf]) -> Self {
        let mut view = self.with_project_state(self.state.clone());
        view.configuration_admission.extend(roots.iter().cloned());
        view
    }

    /// Route a continuation only after its governing root was explicitly checked.
    #[cfg(feature = "native")]
    pub(super) async fn for_document_admitted(&self, uri: &Url) -> DocumentServer<'_> {
        let authored_revision = self.state.documents.stable_revision();
        if uri.to_file_path().is_err() {
            return DocumentServer::primary(self);
        }
        let state = self
            .state
            .document_project_state_async(uri)
            .await
            .unwrap_or_else(|| self.state.clone());
        self.queue_configuration_watches();
        DocumentServer {
            owner: DocumentOwner::Project(self.with_project_state(state)),
            authored_revision,
        }
    }
}

#[cfg(all(test, feature = "native", feature = "experimental-source-navigation"))]
mod tests;

#[cfg(all(test, feature = "native"))]
mod operation_admission_tests;
