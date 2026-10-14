//! Cooperative workspace-symbol request handling.

use std::task::Poll;

use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{SymbolInformation, WorkspaceSymbolParams},
};

#[cfg(not(feature = "native"))]
use crate::ide::WorkspaceSymbolsService;

use super::MaestroServer;

pub(super) async fn search(
    server: &MaestroServer,
    params: &WorkspaceSymbolParams,
) -> Result<Option<Vec<SymbolInformation>>> {
    #[cfg(not(feature = "native"))]
    if !server.state.lsp_features().workspace_symbols {
        return Ok(None);
    }

    yield_to_pending_cancellation().await;
    #[cfg(feature = "native")]
    let symbols = server
        .project_request(async {
            let owner = server.state.clone();
            Ok(server
                .state
                .search_workspace_project_symbols_filtered(&params.query, move |uri| {
                    owner
                        .document_project_state(uri)
                        .unwrap_or_else(|| owner.current_primary_project_state())
                        .lsp_features()
                        .workspace_symbols
                })
                .await)
        })
        .await?;
    #[cfg(not(feature = "native"))]
    let symbols = WorkspaceSymbolsService::search(&server.state, &params.query);

    if symbols.is_empty() {
        Ok(None)
    } else {
        Ok(Some(symbols))
    }
}

/// Give tower-lsp's cancellation layer a poll boundary before the synchronous
/// workspace scan. Editors commonly replace symbol queries while typing, so a
/// queued `$/cancelRequest` should retire the old request before that work.
async fn yield_to_pending_cancellation() {
    let mut yielded = false;
    futures::future::poll_fn(|context| {
        if yielded {
            Poll::Ready(())
        } else {
            yielded = true;
            context.waker().wake_by_ref();
            Poll::Pending
        }
    })
    .await;
}
