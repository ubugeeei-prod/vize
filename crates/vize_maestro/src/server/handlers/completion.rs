//! Single-snapshot completion routing for Vue language requests.

use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{CompletionParams, CompletionResponse},
};

use crate::ide::{CompletionService, IdeContext};

use super::super::project_contexts::DocumentServer;

enum PreparedCompletion<'a> {
    /// A source-resident answer that must not enter the native transaction.
    Response(Option<CompletionResponse>),
    /// The one authored snapshot retained for the type-aware backend.
    Backend(IdeContext<'a>),
}

impl DocumentServer<'_> {
    /// Route one public completion request through a single Vue context.
    pub(super) async fn completion_response(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        if crate::utils::is_jsx_path(params.text_document_position.text_document.uri.path()) {
            if let Some(response) = self.local_resident_component_completion_response(&params)? {
                return self.authored_response(Ok(response));
            }
            if let Some(response) = self.local_completion_response(&params)? {
                return self.authored_response(Ok(response));
            }
            return self.native_request(self.completion_request(params)).await;
        }

        let ctx = match self.prepare_completion(&params)? {
            PreparedCompletion::Response(response) => {
                return self.authored_response(Ok(response));
            }
            PreparedCompletion::Backend(ctx) => ctx,
        };
        self.native_request(self.prepared_completion_request(ctx))
            .await
    }

    /// Build one completion context, then consume every source-only fast path.
    ///
    /// Component props after `:`, stylesheet metadata, and static script object
    /// members now reuse the same descriptor, block, virtual-document, and
    /// buffer snapshots that continue into the backend on a miss.
    fn prepare_completion<'request>(
        &'request self,
        params: &'request CompletionParams,
    ) -> Result<PreparedCompletion<'request>> {
        self.local_request(|| {
            if !self.state.lsp_features().completion {
                return Ok(PreparedCompletion::Response(None));
            }

            let request = &params.text_document_position;
            let uri = &request.text_document.uri;
            let Some(ctx) = IdeContext::at_completion_position(&self.state, uri, request.position)
            else {
                return Ok(PreparedCompletion::Response(None));
            };

            if params
                .context
                .as_ref()
                .and_then(|context| context.trigger_character.as_deref())
                == Some(":")
                && let Some(response) = CompletionService::complete_resident_component_props(&ctx)
            {
                return Ok(PreparedCompletion::Response(Some(response)));
            }

            if ctx.is_in_style() {
                return Ok(PreparedCompletion::Response(CompletionService::complete(
                    &ctx,
                )));
            }
            #[cfg(feature = "native")]
            if let Some(response) = CompletionService::complete_static_object_member(&ctx) {
                return Ok(PreparedCompletion::Response(Some(response)));
            }
            Ok(PreparedCompletion::Backend(ctx))
        })
    }

    /// Continue after native admission without rebuilding authored SFC state.
    async fn prepared_completion_request<'request>(
        &'request self,
        ctx: IdeContext<'request>,
    ) -> Result<Option<CompletionResponse>> {
        #[cfg(feature = "native")]
        let mut ctx = ctx.with_foreground_preparation(self.foreground_preparation());

        #[cfg(feature = "native")]
        {
            let stamp = self.state.corsa_request_stamp();
            let (corsa_bridge, initial) = self
                .state
                .get_corsa_bridge_for_canonical_source(ctx.uri, &ctx.content)
                .await;
            ctx.attach_initial_canonical_document(corsa_bridge.as_ref(), initial, stamp);
            if let Some(response) = CompletionService::complete_with_corsa(&ctx, corsa_bridge).await
            {
                return Ok(Some(response));
            }
        }

        #[cfg(not(feature = "native"))]
        if let Some(response) = CompletionService::complete(&ctx) {
            return Ok(Some(response));
        }

        if ctx.block_type.is_some() {
            return Ok(None);
        }

        let items = self.get_block_snippets();
        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CompletionResponse::Array(items)))
        }
    }
}
