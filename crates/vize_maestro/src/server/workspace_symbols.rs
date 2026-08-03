//! Cooperative workspace-symbol request handling.

use std::task::Poll;

use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{SymbolInformation, WorkspaceSymbolParams},
};

use crate::ide::WorkspaceSymbolsService;

use super::MaestroServer;

pub(super) async fn search(
    server: &MaestroServer,
    params: &WorkspaceSymbolParams,
) -> Result<Option<Vec<SymbolInformation>>> {
    if !server.state.lsp_features().workspace_symbols {
        return Ok(None);
    }

    yield_to_pending_cancellation().await;
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
