//! Keep native open/query/map sequences together while the executor can yield.

use std::future::Future;
#[cfg(feature = "native")]
use tower_lsp::jsonrpc::Error;
use tower_lsp::jsonrpc::Result;

use super::MaestroServer;

impl MaestroServer {
    /// Bulk requests do not borrow the retired primary's Corsa transaction.
    /// Each native owner acquires its own scope; the aggregate also refuses
    /// results if any routing policy or shared authored buffer changed.
    #[cfg(feature = "native")]
    pub(super) async fn project_request<T>(
        &self,
        request: impl Future<Output = Result<T>>,
    ) -> Result<T> {
        if self.state.project_routing_shutting_down() {
            return Err(Error::content_modified());
        }
        let generation = self
            .state
            .stable_project_routing_generation()
            .ok_or_else(Error::content_modified)?;
        let documents = self
            .state
            .documents
            .stable_revision()
            .ok_or_else(Error::content_modified)?;
        let result = request.await;
        if self.state.project_routing_shutting_down()
            || self.state.stable_project_routing_generation() != Some(generation)
            || self.state.documents.stable_revision() != Some(documents)
        {
            return Err(Error::content_modified());
        }
        result
    }

    pub(super) async fn native_request<T>(
        &self,
        request: impl Future<Output = Result<T>>,
    ) -> Result<T> {
        #[cfg(feature = "native")]
        if self.state.project_context_retired() {
            return Err(Error::content_modified());
        }
        #[cfg(feature = "native")]
        let scope = if self.state.is_lsp_typecheck_enabled() {
            Some(self.state.corsa_request_scope().await)
        } else {
            None
        };
        let reply = request.await;
        #[cfg(feature = "native")]
        if self.state.project_context_retired() {
            return Err(Error::content_modified());
        }
        #[cfg(feature = "native")]
        if let Some(scope) = scope
            && !scope.is_current()
        {
            scope.trace_refusal();
            return Err(Error::content_modified());
        }
        reply
    }
}

/// The real editor fixture opts into failure provenance; ordinary requests
/// never consult this flag unless a complete result has already been refused.
#[cfg(feature = "native")]
pub(super) fn trace_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("VIZE_LSP_TRACE_NATIVE_SCOPE").as_deref()
            == Some(std::ffi::OsStr::new("1"))
    })
}

#[cfg(feature = "native")]
pub(super) fn trace_watched_file_events(
    params: &tower_lsp::lsp_types::DidChangeWatchedFilesParams,
) {
    if trace_enabled() {
        tracing::info!(
            target: "vize_maestro::server::workspace_files",
            changes = ?params.changes,
            "incoming watched file events"
        );
    }
}

#[cfg(all(test, feature = "native"))]
mod tests;
