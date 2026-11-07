//! Single-snapshot hover routing for the public LSP request.

use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{Hover, HoverParams, Position},
};

use crate::ide::{HoverService, IdeContext};

use super::super::{MaestroServer, project_contexts::DocumentServer};

pub(super) enum PreparedHover<'a> {
    /// A source-resident answer that must not enter the native transaction.
    Response(Option<Hover>),
    /// The one authored snapshot retained for the type-aware backend.
    Backend(IdeContext<'a>),
}

impl DocumentServer<'_> {
    /// Build the request context once, then consume every source-only fast path.
    ///
    /// Returning the context keeps descriptor, block, virtual-document, and
    /// buffer snapshots alive for the backend instead of rebuilding them after
    /// the local probe. Markup and stylesheet answers also return before Corsa
    /// transaction admission, which matters when a large project is busy.
    pub(super) fn prepare_hover<'request>(
        &'request self,
        params: &'request HoverParams,
    ) -> Result<PreparedHover<'request>> {
        self.local_request(|| {
            if !self.state.lsp_features().hover {
                return Ok(PreparedHover::Response(None));
            }

            let request = &params.text_document_position_params;
            let uri = &request.text_document.uri;
            let position = request.position;
            let Some(ctx) = IdeContext::at_position(&self.state, uri, position) else {
                return Ok(PreparedHover::Response(None));
            };

            let response = if ctx.is_in_style() {
                HoverService::hover(&ctx).or_else(|| super::super::annotations::color_hover(&ctx))
            } else {
                #[cfg(feature = "native")]
                if let Some(response) = HoverService::hover_resident_markup(&ctx) {
                    Some(response)
                } else {
                    return Ok(PreparedHover::Backend(ctx));
                }
                #[cfg(not(feature = "native"))]
                return Ok(PreparedHover::Backend(ctx));
            };

            let response = if let Some(lint) = self.get_lint_hover_at_position(uri, position) {
                Some(MaestroServer::merge_hover_with_lint(response, lint))
            } else {
                response
            };
            Ok(PreparedHover::Response(response))
        })
    }

    /// Continue a prepared hover after native admission without reading the
    /// authored document, descriptor, block, or virtual documents again.
    pub(super) async fn prepared_hover_request<'request>(
        &'request self,
        ctx: IdeContext<'request>,
        position: Position,
    ) -> Result<Option<Hover>> {
        #[cfg(feature = "native")]
        let mut ctx = ctx.with_foreground_preparation(self.foreground_preparation());

        #[cfg(feature = "native")]
        let mut response = {
            let stamp = self.state.corsa_request_stamp();
            let (corsa_bridge, initial) = self
                .state
                .get_corsa_bridge_for_canonical_source(ctx.uri, &ctx.content)
                .await;
            ctx.attach_initial_canonical_document(corsa_bridge.as_ref(), initial, stamp);
            let response = HoverService::hover_with_corsa(&ctx, corsa_bridge).await;
            self.state
                .cache_native_hover(ctx.uri, position, stamp, response.clone());
            response
        };

        #[cfg(not(feature = "native"))]
        let mut response =
            HoverService::hover(&ctx).or_else(|| super::super::annotations::color_hover(&ctx));

        if let Some(lint) = self.get_lint_hover_at_position(ctx.uri, position) {
            response = Some(MaestroServer::merge_hover_with_lint(response, lint));
        }
        Ok(response)
    }
}
