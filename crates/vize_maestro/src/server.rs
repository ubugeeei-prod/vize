//! LSP server implementation.
//!
//! This module contains the core LSP server using tower-lsp.

use std::{
    sync::Arc,
    task::{Context, Poll},
};

use futures::{FutureExt, channel::oneshot, future::BoxFuture};
use parking_lot::Mutex;
use tower::Service;
use tower_lsp::jsonrpc::{Request, Response};

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

/// Transport adapter that reports when the LSP `exit` notification has been
/// fully handled. `tower-lsp` stops accepting requests after `exit`, but its
/// transport continues waiting for stdin EOF. Editors keep that pipe open
/// while waiting for the child to terminate, so the process would otherwise
/// deadlock with its client.
pub(crate) struct ExitAwareService<S> {
    inner: S,
    exit_sender: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

pub(crate) fn with_exit_signal<S>(inner: S) -> (ExitAwareService<S>, oneshot::Receiver<()>) {
    let (exit_sender, exit_receiver) = oneshot::channel();
    (
        ExitAwareService {
            inner,
            exit_sender: Arc::new(Mutex::new(Some(exit_sender))),
        },
        exit_receiver,
    )
}

impl<S> Service<Request> for ExitAwareService<S>
where
    S: Service<Request, Response = Option<Response>> + Send + 'static,
    S::Error: Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, context: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(context)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let is_exit = request.method() == "exit";
        let response = self.inner.call(request);
        let exit_sender = Arc::clone(&self.exit_sender);

        async move {
            let result = response.await;
            if is_exit && let Some(sender) = exit_sender.lock().take() {
                let _ = sender.send(());
            }
            result
        }
        .boxed()
    }
}
