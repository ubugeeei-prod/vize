//! Client routing hints derived from the resolved workspace configuration.

use tower_lsp::lsp_types::ServerCapabilities;

use super::{MaestroServer, server_capabilities};

impl MaestroServer {
    pub(super) fn client_capabilities(&self) -> ServerCapabilities {
        let mut capabilities = server_capabilities(self.state.lsp_features());
        let experimental = capabilities
            .experimental
            .get_or_insert_with(|| serde_json::json!({}));
        experimental["vize"] = serde_json::json!({
            "jsxTypecheck": cfg!(feature = "native")
                && self.state.is_lsp_typecheck_enabled()
                && self.state.jsx_typecheck_enabled()
        });
        capabilities
    }
}
