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
        if scope.is_some_and(|scope| !scope.is_current()) {
            return Err(Error::content_modified());
        }
        reply
    }
}

#[cfg(all(test, feature = "native"))]
mod tests;
