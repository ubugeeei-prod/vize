//! LSP server implementation.
//!
//! This module contains the core LSP server using tower-lsp.

use std::{
    future::Future,
    task::{Context, Poll},
};

use futures::{FutureExt, StreamExt, channel::mpsc, future::BoxFuture};
use tower::Service;
use tower_lsp::jsonrpc::{Request, Response};

mod annotations;
mod auto_insert;
mod capabilities;
mod code_actions;
mod diagnostic_publishing;
mod document_structure;
mod format;
mod handlers;
mod helpers;
mod importers;
#[cfg(feature = "native")]
mod initial_diagnostics;
mod state;
mod workspace_files;
mod workspace_folder_events;
mod workspace_symbols;

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
    #[allow(clippy::disallowed_types)] // Shared only with the single diagnostics worker.
    state: std::sync::Arc<ServerState>,
    /// Single background lane for the type-diagnostic work scheduled by
    /// `didOpen`. Keeping the sender on the foreground server lets the worker
    /// own the same client and state without spawning one thread per document.
    #[cfg(feature = "native")]
    initial_diagnostics: Option<initial_diagnostics::InitialDiagnosticsScheduler>,
}

impl MaestroServer {
    /// Create a new Maestro server instance.
    #[allow(clippy::disallowed_types)] // One diagnostics worker shares the live server state.
    pub fn new(client: Client) -> Self {
        let state = std::sync::Arc::new(ServerState::new());

        #[cfg(feature = "native")]
        let initial_diagnostics = {
            // The worker must not retain another sender, otherwise dropping the
            // foreground server could never close its queue.
            let worker = Self {
                client: client.clone(),
                state: state.clone(),
                initial_diagnostics: None,
            };
            Some(initial_diagnostics::InitialDiagnosticsScheduler::new(
                worker,
            ))
        };

        Self {
            client,
            state,
            #[cfg(feature = "native")]
            initial_diagnostics,
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
    exit_sender: mpsc::UnboundedSender<()>,
}

pub(crate) fn with_exit_signal<S>(
    inner: S,
) -> (ExitAwareService<S>, impl Future<Output = ()> + Send) {
    let (exit_sender, mut exit_receiver) = mpsc::unbounded();
    (ExitAwareService { inner, exit_sender }, async move {
        let _ = exit_receiver.next().await;
    })
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
        let exit_sender = self.exit_sender.clone();

        async move {
            let result = response.await;
            if is_exit {
                let _ = exit_sender.unbounded_send(());
            }
            result
        }
        .boxed()
    }
}
