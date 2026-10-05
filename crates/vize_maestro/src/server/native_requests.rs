//! Keep native open/query/map sequences together while the executor can yield.

use std::future::Future;
#[cfg(feature = "native")]
use tower_lsp::jsonrpc::Error;
use tower_lsp::jsonrpc::Result;

use super::MaestroServer;

impl MaestroServer {
    pub(super) async fn native_request<T>(
        &self,
        request: impl Future<Output = Result<T>>,
    ) -> Result<T> {
        #[cfg(feature = "native")]
        let scope = if self.state.is_lsp_typecheck_enabled() {
            Some(self.state.corsa_request_scope().await)
        } else {
            None
        };
        let reply = request.await;
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

#[cfg(all(test, feature = "native"))]
mod tests;
