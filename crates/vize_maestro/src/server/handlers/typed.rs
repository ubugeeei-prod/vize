//! Owned-source type-aware hover, completion and rename handlers.

use super::{MaestroServer, Result};
use crate::ide::{CompletionService, HoverService, IdeContext, RenameService};
use tower_lsp::lsp_types::{
    CompletionParams, CompletionResponse, Hover, HoverParams, PrepareRenameResponse, RenameParams,
    TextDocumentPositionParams, WorkspaceEdit,
};

impl MaestroServer {
    pub(super) fn local_hover_response(
        &self,
        params: &HoverParams,
    ) -> Result<Option<Option<Hover>>> {
        self.local_request(|| {
            if !self.state.lsp_features().hover {
                return Ok(Some(None));
            }
            let uri = &params.text_document_position_params.text_document.uri;
            if crate::utils::is_jsx_path(uri.path()) {
                return Ok(None);
            }
            let position = params.text_document_position_params.position;
            let Some(ctx) = IdeContext::at_position(&self.state, uri, position) else {
                return Ok(Some(None));
            };
            if !ctx.is_in_style() {
                return Ok(None);
            }
            let mut response =
                HoverService::hover(&ctx).or_else(|| super::super::annotations::color_hover(&ctx));
            if let Some(lint) = self.get_lint_hover_at_position(uri, position) {
                response = Some(Self::merge_hover_with_lint(response, lint));
            }
            Ok(Some(response))
        })
    }

    pub(in crate::server) fn local_completion_response(
        &self,
        params: &CompletionParams,
    ) -> Result<Option<Option<CompletionResponse>>> {
        self.local_request(|| {
            if !self.state.lsp_features().completion {
                return Ok(Some(None));
            }
            let request = &params.text_document_position;
            let uri = &request.text_document.uri;
            if crate::utils::is_jsx_path(uri.path()) {
                return Ok(None);
            }
            let position = request.position;
            let Some(ctx) = IdeContext::at_completion_position(&self.state, uri, position) else {
                return Ok(Some(None));
            };
            if ctx.is_in_style() {
                return Ok(Some(CompletionService::complete(&ctx)));
            }
            #[cfg(feature = "native")]
            if let Some(response) = CompletionService::complete_static_object_member(&ctx) {
                return Ok(Some(Some(response)));
            }
            Ok(None)
        })
    }

    async fn hover_request_with_preparation(
        &self,
        params: HoverParams,
        preparation: Option<(&tower_lsp::lsp_types::Url, &std::sync::atomic::AtomicBool)>,
    ) -> Result<Option<Hover>> {
        #[cfg(not(feature = "native"))]
        let _ = preparation;
        if !self.state.lsp_features().hover {
            return Ok(None);
        }

        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Some(ctx) = IdeContext::at_position(&self.state, uri, position) else {
            return Ok(None);
        };
        #[cfg(feature = "native")]
        let ctx = ctx.with_foreground_preparation(preparation);

        // Type-aware hover for `.jsx`/`.tsx` (opt-in `typeChecker.jsxTypecheck`).
        // Routed before the SFC path since JSX documents never produce an SFC
        // block type. React `.tsx` is untouched when the flag is off.
        #[cfg(feature = "native")]
        if crate::utils::is_jsx_path(uri.path()) {
            if self.state.jsx_typecheck_enabled() {
                if let Some(hover) = self.jsx_intrinsic_hover(&ctx, position).await {
                    return Ok(Some(hover));
                }
                let corsa_bridge = self.state.get_corsa_bridge().await;
                return Ok(crate::ide::JsxService::hover(&ctx, corsa_bridge).await);
            }
            return Ok(None);
        }

        #[cfg(feature = "native")]
        let mut hover_result: Option<Hover> = if ctx.is_in_style() {
            HoverService::hover(&ctx).or_else(|| super::super::annotations::color_hover(&ctx))
        } else {
            let corsa_bridge = self.state.get_corsa_bridge().await;
            HoverService::hover_with_corsa(&ctx, corsa_bridge).await
        };

        #[cfg(not(feature = "native"))]
        let mut hover_result: Option<Hover> =
            HoverService::hover(&ctx).or_else(|| super::super::annotations::color_hover(&ctx));

        let lint_hover = self.get_lint_hover_at_position(uri, position);
        if let Some(lint_info) = lint_hover {
            hover_result = Some(Self::merge_hover_with_lint(hover_result, lint_info));
        }

        Ok(hover_result)
    }

    async fn completion_request_with_preparation(
        &self,
        params: CompletionParams,
        preparation: Option<(&tower_lsp::lsp_types::Url, &std::sync::atomic::AtomicBool)>,
    ) -> Result<Option<CompletionResponse>> {
        #[cfg(not(feature = "native"))]
        let _ = preparation;
        if !self.state.lsp_features().completion {
            return Ok(None);
        }

        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let Some(ctx) = IdeContext::at_completion_position(&self.state, uri, position) else {
            return Ok(None);
        };
        #[cfg(feature = "native")]
        let ctx = ctx.with_foreground_preparation(preparation);
        // JSX completion is opt-in so React remains untouched.
        #[cfg(feature = "native")]
        if crate::utils::is_jsx_path(uri.path()) {
            if self.state.jsx_typecheck_enabled() {
                let corsa_bridge = self.state.get_corsa_bridge().await;
                if let Some(response) = crate::ide::JsxService::completion(&ctx, corsa_bridge).await
                {
                    return Ok(Some(response));
                }
            }
            return Ok(None);
        }

        // CSS uses only resident block context and static metadata, never Corsa.
        if ctx.is_in_style() {
            return Ok(CompletionService::complete(&ctx));
        }

        #[cfg(feature = "native")]
        {
            if let Some(response) = CompletionService::complete_static_object_member(&ctx) {
                return Ok(Some(response));
            }
            let corsa_bridge = self.state.get_corsa_bridge().await;
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

    pub(super) async fn prepare_rename_request(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        if !self.state.lsp_features().rename {
            return Ok(None);
        }

        let uri = &params.text_document.uri;
        let position = params.position;

        let Some(ctx) = IdeContext::at_position(&self.state, uri, position) else {
            return Ok(None);
        };

        // Type-aware prepare-rename for `.jsx`/`.tsx` (opt-in `typeChecker.jsxTypecheck`).
        #[cfg(feature = "native")]
        if crate::utils::is_jsx_path(uri.path()) {
            if self.state.jsx_typecheck_enabled() {
                let corsa_bridge = self.state.get_corsa_bridge().await;
                return Ok(crate::ide::JsxRenameService::prepare_rename(&ctx, corsa_bridge).await);
            }
            return Ok(None);
        }

        #[cfg(feature = "native")]
        {
            let corsa_bridge = self.state.get_corsa_bridge().await;
            Ok(RenameService::prepare_rename_with_corsa(&ctx, corsa_bridge).await)
        }

        #[cfg(not(feature = "native"))]
        {
            Ok(RenameService::prepare_rename(&ctx))
        }
    }

    pub(super) async fn rename_request(
        &self,
        params: RenameParams,
    ) -> Result<Option<WorkspaceEdit>> {
        if !self.state.lsp_features().rename {
            return Ok(None);
        }

        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = &params.new_name;

        let Some(ctx) = IdeContext::at_position(&self.state, uri, position) else {
            return Ok(None);
        };

        // Type-aware rename for `.jsx`/`.tsx` (opt-in `typeChecker.jsxTypecheck`).
        #[cfg(feature = "native")]
        if crate::utils::is_jsx_path(uri.path()) {
            if self.state.jsx_typecheck_enabled() {
                let corsa_bridge = self.state.get_corsa_bridge().await;
                return Ok(
                    crate::ide::JsxRenameService::rename(&ctx, new_name, corsa_bridge).await,
                );
            }
            return Ok(None);
        }

        #[cfg(feature = "native")]
        {
            let corsa_bridge = self.state.get_corsa_bridge().await;
            Ok(RenameService::rename_with_corsa(&ctx, new_name, corsa_bridge).await)
        }

        #[cfg(not(feature = "native"))]
        {
            Ok(RenameService::rename(&ctx, new_name))
        }
    }
}

impl super::super::project_contexts::DocumentServer<'_> {
    pub(super) async fn hover_request(&self, params: HoverParams) -> Result<Option<Hover>> {
        MaestroServer::hover_request_with_preparation(self, params, self.foreground_preparation())
            .await
    }

    pub(super) async fn completion_request(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        MaestroServer::completion_request_with_preparation(
            self,
            params,
            self.foreground_preparation(),
        )
        .await
    }
}
