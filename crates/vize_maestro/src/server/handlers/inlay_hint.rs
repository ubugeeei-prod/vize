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
        #[cfg(feature = "native")]
        let (content, builtin_types, native_hints) = if self.state.is_lsp_typecheck_enabled() {
            let ctx = crate::ide::IdeContext::with_content(&self.state, uri, 0, content);
            let (builtin_types, hints) = if let Some(bridge) = self.state.get_corsa_bridge().await {
                (
                    false,
                    InlayHintService::native_hints(&ctx, &bridge, params.range).await,
                )
            } else {
                (true, Vec::new())
            };
            (ctx.content, builtin_types, hints)
        } else {
            (content, true, Vec::new())
        };
        #[cfg(not(feature = "native"))]
        let builtin_types = true;
        let mut hints =
            InlayHintService::decorations(&self.state, &content, uri, params.range, builtin_types);
        #[cfg(feature = "native")]
        hints.extend(native_hints);
        hints.sort_by_key(|hint| (hint.position.line, hint.position.character));
        Ok((!hints.is_empty()).then_some(hints))
    }
}
