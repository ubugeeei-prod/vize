//! Keep native open/query/map sequences together while the executor can yield.

use std::future::Future;
#[cfg(feature = "native")]
use tower_lsp::jsonrpc::Error;
use tower_lsp::jsonrpc::Result;

use super::MaestroServer;

impl super::project_contexts::DocumentServer<'_> {
    /// Preserve native transaction/cancellation ordering, then refuse the whole
    /// reply if authored coordinates became stale during admission or native work.
    async fn authored_request<T>(&self, request: impl Future<Output = Result<T>>) -> Result<T> {
        self.authored_response(request.await)
    }

    /// Only completed answers pass this boundary. A local probe returning no
    /// answer must retain the native transaction's existing ordering.
    pub(super) fn authored_response<T>(&self, result: Result<T>) -> Result<T> {
        #[cfg(feature = "native")]
        if !self.authored_revision_is_current() {
            return Err(Error::content_modified());
        }
        result
    }

    pub(super) async fn native_request<T>(
        &self,
        request: impl Future<Output = Result<T>>,
    ) -> Result<T> {
        self.authored_request(MaestroServer::native_request(self, request))
            .await
    }

    pub(super) async fn workspace_native_request<T, F, Fut>(&self, request: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        self.authored_request(MaestroServer::workspace_native_request(self, request))
            .await
    }
}

impl MaestroServer {
    /// Source-local answers never touch the backend or borrow its transaction.
    /// Retain the same source/config freshness boundary around their sync work.
    pub(super) fn local_request<T>(&self, request: impl FnOnce() -> Result<T>) -> Result<T> {
        #[cfg(feature = "native")]
        let stamp = self.state.corsa_request_stamp();
        #[cfg(feature = "native")]
        if !stamp.is_current(&self.state) {
            return Err(Error::content_modified());
        }
        let response = request();
        #[cfg(feature = "native")]
        if !stamp.is_current(&self.state) {
            return Err(Error::content_modified());
        }
        response
    }

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

impl MaestroServer {
    /// Run a request that may open the whole workspace. Building that
    /// projection inside the native transaction keeps every other native
    /// request waiting, so the first attempt gives the transaction back as
    /// soon as it needs an unprepared projection; the projection is built on
    /// its own thread and the request starts over inside a new transaction.
    /// The second attempt never defers, so a request always completes.
    pub(super) async fn workspace_native_request<T, F, Fut>(&self, request: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        #[cfg(feature = "native")]
        if self.state.project_context_retired() {
            return Err(Error::content_modified());
        }
        #[cfg(feature = "native")]
        if self.state.is_lsp_typecheck_enabled() && !inline_workspace_build() {
            let scope = self.state.corsa_request_scope().await;
            let armed = self.state.workspace_preparation.arm();
            let first = {
                let mut attempt = std::pin::pin!(request());
                std::future::poll_fn(|cx| {
                    if let std::task::Poll::Ready(reply) = attempt.as_mut().poll(cx) {
                        return std::task::Poll::Ready(Ok(reply));
                    }
                    match self.state.workspace_preparation.take() {
                        Some(preparation) => std::task::Poll::Ready(Err(preparation)),
                        None => std::task::Poll::Pending,
                    }
                })
                .await
            };
            drop(armed);
            match first {
                Ok(reply) => {
                    if self.state.project_context_retired() {
                        return Err(Error::content_modified());
                    }
                    if !scope.is_current() {
                        scope.trace_refusal();
                        return Err(Error::content_modified());
                    }
                    return reply;
                }
                Err(preparation) => {
                    drop(scope);
                    preparation.await;
                }
            }
        }
        self.native_request(request()).await
    }
}

/// Operators can keep workspace projections on the request thread.
#[cfg(feature = "native")]
fn inline_workspace_build() -> bool {
    static INLINE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *INLINE.get_or_init(|| {
        std::env::var_os("VIZE_LSP_INLINE_WORKSPACE_BUILD").as_deref()
            == Some(std::ffi::OsStr::new("1"))
    })
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

#[cfg(all(test, feature = "native"))]
mod workspace_tests;
