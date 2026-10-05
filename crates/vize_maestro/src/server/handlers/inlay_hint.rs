//! Owned-buffer hints within the existing native request generation scope.

use super::{MaestroServer, Result};
use crate::ide::InlayHintService;
use tower_lsp::lsp_types::{InlayHint, InlayHintParams};

#[cfg(test)]
mod tests;

impl MaestroServer {
    pub(super) async fn inlay_hint_request(
        &self,
        params: InlayHintParams,
    ) -> Result<Option<Vec<InlayHint>>> {
        if !self.state.lsp_features().inlay_hints {
            return Ok(None);
        }
        let uri = &params.text_document.uri;
        let Some(content) = self.state.documents.text(uri) else {
            return Ok(None);
        };
        let mut hints = InlayHintService::decorations(&self.state, &content, uri, params.range);
        #[cfg(feature = "native")]
        if self.state.is_lsp_typecheck_enabled() {
            let ctx = crate::ide::IdeContext::with_content(&self.state, uri, 0, content);
            if let Some(bridge) = self.state.get_corsa_bridge().await {
                hints.extend(InlayHintService::native_hints(&ctx, &bridge, params.range).await);
            }
        }
        hints.sort_by_key(|hint| (hint.position.line, hint.position.character));
        Ok((!hints.is_empty()).then_some(hints))
    }
}
