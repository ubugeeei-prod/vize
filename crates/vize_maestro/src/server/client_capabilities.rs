//! Client routing hints derived from the resolved workspace configuration.

use tower_lsp::lsp_types::ServerCapabilities;

use super::{MaestroServer, server_capabilities};

impl MaestroServer {
    pub(super) fn client_capabilities(&self) -> ServerCapabilities {
        #[cfg(feature = "native")]
        let features = self.state.project_capability_features();
        #[cfg(not(feature = "native"))]
        let features = self.state.lsp_features();
        #[cfg(feature = "native")]
        let jsx_typecheck = features.typecheck && self.state.project_jsx_typecheck_enabled();
        #[cfg(not(feature = "native"))]
        let jsx_typecheck = false;
        let mut capabilities = server_capabilities(features);
        let experimental = capabilities
            .experimental
            .get_or_insert_with(|| serde_json::json!({}));
        experimental["vize"] = serde_json::json!({
            "jsxTypecheck": jsx_typecheck
        });
        capabilities
    }
}
