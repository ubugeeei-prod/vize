//! LSP protocol handler implementations.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use tower_lsp::{
    LanguageServer,
    jsonrpc::Result,
    lsp_types::{
        CodeActionParams, CodeActionResponse, CodeLens, CodeLensParams, ColorInformation,
        ColorPresentation, ColorPresentationParams, CompletionItem, CompletionParams,
        CompletionResponse, CreateFilesParams, DeleteFilesParams, DidChangeConfigurationParams,
        DidChangeTextDocumentParams, DidChangeWatchedFilesParams, DidChangeWorkspaceFoldersParams,
        DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
        DocumentColorParams, DocumentFormattingParams, DocumentHighlight, DocumentHighlightParams,
        DocumentLink, DocumentLinkParams, DocumentOnTypeFormattingParams,
        DocumentRangeFormattingParams, DocumentSymbolParams, DocumentSymbolResponse, FoldingRange,
        FoldingRangeParams, Hover, HoverParams, InitializeParams, InitializeResult,
        InitializedParams, InlayHint, InlayHintParams, LinkedEditingRangeParams,
        LinkedEditingRanges, Location, PrepareRenameResponse, ReferenceParams, RenameFilesParams,
        RenameParams, SelectionRange, SelectionRangeParams, SemanticTokensParams,
        SemanticTokensRangeParams, SemanticTokensRangeResult, SemanticTokensResult, ServerInfo,
        SymbolInformation, TextDocumentPositionParams, TextEdit, WorkspaceEdit,
        WorkspaceSymbolParams,
    },
};

// Test modules still construct positions and ranges through `use super::*`.
#[cfg(test)]
use tower_lsp::lsp_types::{Position, Range};

use super::{MaestroServer, server_capabilities};
use crate::ide::{
    CompletionService, DocumentHighlightService, DocumentLinkService, HoverService, IdeContext,
    ReferencesService, RenameService, SemanticTokensService, position_to_offset,
};

mod call_hierarchy;
mod navigation;
mod signature_help;
use call_hierarchy::{
    CHIncomingParams, CHIncomingResponse, CHItems, CHOutgoingParams, CHOutgoingResponse,
    CHPrepareParams,
};
use navigation::{
    DeclParams, DeclResponse, DefParams, DefResponse, ImplParams, ImplResponse, TypeDefParams,
    TypeDefResponse,
};
use signature_help::{SigHelp, SigHelpParams};

#[tower_lsp::async_trait]
impl LanguageServer for MaestroServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        super::workspace_files::record_watcher_support(&self.state, &params.capabilities);
        // Resolve workspace root
        let workspace_path = self.state.primary_workspace_path(&params);

        // Load format config from workspace root (always, regardless of feature)
        if let Some(ref path) = workspace_path {
            self.state.load_workspace_config(path);
        }

        // Record every workspace folder so per-document features resolve their own folder's config in multi-root sessions (#3240).
        self.state
            .apply_initialize_workspace_folders(params.workspace_folders.as_deref());

        self.state
            .apply_lsp_initialization_options(params.initialization_options.as_ref());

        // Set workspace root for native features (Corsa, batch checker)
        #[cfg(feature = "native")]
        if let Some(path) = workspace_path {
            tracing::info!("Setting workspace root: {:?}", path);
            self.state.set_workspace_root(path);
        }

        Ok(InitializeResult {
            capabilities: server_capabilities(self.state.lsp_features()),
            server_info: Some(ServerInfo {
                name: "vize-maestro".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        super::workspace_files::initialized(self).await;
    }

    async fn did_change_configuration(&self, _params: DidChangeConfigurationParams) {
        tracing::debug!(
            "Received workspace/didChangeConfiguration; VS Code restarts the server for Vize configuration changes"
        );
    }

    // Keep the per-folder configuration contexts in sync when the editor adds or removes roots mid-session (#3240).
    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        self.reconfigure_workspace_folders(&params.event).await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.open_document(params).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;
        self.apply_document_changes(&uri, params.content_changes, version)
            .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        self.publish_diagnostics(&uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.state.close_document(&uri);

        // Clean up virtual documents cache
        self.state.remove_virtual_docs(&uri);

        // Clear diagnostics
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        if !self.state.lsp_features().hover {
            return Ok(None);
        }

        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Some(content) = self.state.documents.text(uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&content, position.line, position.character) else {
            return Ok(None);
        };

        let ctx = IdeContext::with_content(&self.state, uri, offset, content);

        // Type-aware hover for `.jsx`/`.tsx` (opt-in `typeChecker.jsxTypecheck`).
        // Routed before the SFC path since JSX documents never produce an SFC
        // block type. React `.tsx` is untouched when the flag is off.
        #[cfg(feature = "native")]
        if crate::utils::is_jsx_path(uri.path()) {
            if self.state.jsx_typecheck_enabled() {
                let corsa_bridge = self.state.get_corsa_bridge().await;
                return Ok(crate::ide::JsxService::hover(&ctx, corsa_bridge).await);
            }
            return Ok(None);
        }

        #[cfg(feature = "native")]
        let mut hover_result: Option<Hover> = {
            let corsa_bridge = self.state.get_corsa_bridge().await;
            HoverService::hover_with_corsa(&ctx, corsa_bridge).await
        };

        #[cfg(not(feature = "native"))]
        let mut hover_result: Option<Hover> = HoverService::hover(&ctx);

        let lint_hover = self.get_lint_hover_at_position(uri, &ctx.content, position);
        if let Some(lint_info) = lint_hover {
            hover_result = Some(Self::merge_hover_with_lint(hover_result, lint_info));
        }

        Ok(hover_result)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        if !self.state.lsp_features().completion {
            return Ok(None);
        }

        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let Some(content) = self.state.documents.text(uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&content, position.line, position.character) else {
            return Ok(None);
        };

        let ctx = IdeContext::with_content(&self.state, uri, offset, content);

        // Type-aware completion for `.jsx`/`.tsx` (opt-in
        // `typeChecker.jsxTypecheck`). React `.tsx` is untouched when off.
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

    async fn completion_resolve(&self, item: CompletionItem) -> Result<CompletionItem> {
        Ok(item)
    }

    async fn signature_help(&self, params: SigHelpParams) -> Result<Option<SigHelp>> {
        signature_help::signature_help(self, params).await
    }

    async fn goto_definition(&self, params: DefParams) -> Result<Option<DefResponse>> {
        navigation::goto_definition(self, params).await
    }

    async fn goto_type_definition(&self, params: TypeDefParams) -> Result<Option<TypeDefResponse>> {
        navigation::goto_type_definition(self, params).await
    }

    async fn goto_declaration(&self, params: DeclParams) -> Result<Option<DeclResponse>> {
        navigation::goto_declaration(self, params).await
    }

    async fn goto_implementation(&self, params: ImplParams) -> Result<Option<ImplResponse>> {
        navigation::goto_implementation(self, params).await
    }

    async fn prepare_call_hierarchy(&self, params: CHPrepareParams) -> Result<Option<CHItems>> {
        call_hierarchy::prepare(self, params).await
    }

    async fn incoming_calls(&self, params: CHIncomingParams) -> Result<Option<CHIncomingResponse>> {
        let _ = params;
        Ok(None)
    }

    async fn outgoing_calls(&self, params: CHOutgoingParams) -> Result<Option<CHOutgoingResponse>> {
        let _ = params;
        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        if !self.state.lsp_features().references {
            return Ok(None);
        }

        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;

        let Some(content) = self.state.documents.text(uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&content, position.line, position.character) else {
            return Ok(None);
        };

        let ctx = IdeContext::with_content(&self.state, uri, offset, content);

        // Type-aware references for `.jsx`/`.tsx` (opt-in `typeChecker.jsxTypecheck`).
        // Routed before the SFC path since JSX documents never produce an SFC
        // block type. React `.tsx` is untouched when the flag is off.
        #[cfg(feature = "native")]
        if crate::utils::is_jsx_path(uri.path()) {
            if self.state.jsx_typecheck_enabled() {
                let corsa_bridge = self.state.get_corsa_bridge().await;
                if let Some(locations) = crate::ide::JsxReferencesService::references(
                    &ctx,
                    include_declaration,
                    corsa_bridge,
                )
                .await
                {
                    return Ok(Some(locations));
                }
            }
            return Ok(None);
        }

        #[cfg(feature = "native")]
        {
            let locations = if self.state.lsp_features().cross_file {
                ReferencesService::references_with_corsa(
                    &ctx,
                    include_declaration,
                    self.state.get_corsa_bridge().await,
                )
                .await
            } else {
                ReferencesService::references(&ctx, include_declaration)
            };
            if let Some(locations) = locations {
                return Ok(Some(locations));
            }
        }

        #[cfg(not(feature = "native"))]
        if let Some(locations) = ReferencesService::references(&ctx, include_declaration) {
            return Ok(Some(locations));
        }

        Ok(None)
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        if !self.state.lsp_features().references {
            return Ok(None);
        }

        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Some(content) = self.state.documents.text(uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&content, position.line, position.character) else {
            return Ok(None);
        };

        let ctx = IdeContext::with_content(&self.state, uri, offset, content);

        Ok(DocumentHighlightService::highlights(&ctx))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        if !self.state.lsp_features().document_symbols {
            return Ok(None);
        }
        Ok(super::document_structure::document_symbols(
            &self.state,
            &params,
        ))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        Ok(super::code_actions::code_actions(self, &params))
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        if !self.state.lsp_features().rename {
            return Ok(None);
        }

        let uri = &params.text_document.uri;
        let position = params.position;

        let Some(content) = self.state.documents.text(uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&content, position.line, position.character) else {
            return Ok(None);
        };

        let ctx = IdeContext::with_content(&self.state, uri, offset, content);

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

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        if !self.state.lsp_features().rename {
            return Ok(None);
        }

        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = &params.new_name;

        let Some(content) = self.state.documents.text(uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&content, position.line, position.character) else {
            return Ok(None);
        };

        let ctx = IdeContext::with_content(&self.state, uri, offset, content);

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

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        if !self.state.lsp_features().semantic_tokens {
            return Ok(None);
        }

        let uri = &params.text_document.uri;

        let Some(content) = self.state.documents.text(uri) else {
            return Ok(None);
        };

        // `.jsx`/`.tsx`: highlight the dynamic JSX expressions. Structural, so
        // not gated on `typeChecker.jsxTypecheck`.
        if crate::utils::is_jsx_path(uri.path()) {
            return Ok(crate::ide::JsxSemanticTokensService::tokens(&content, uri));
        }

        Ok(SemanticTokensService::get_tokens(&content, uri))
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        if !self.state.lsp_features().semantic_tokens {
            return Ok(None);
        }

        let uri = &params.text_document.uri;

        let Some(content) = self.state.documents.text(uri) else {
            return Ok(None);
        };

        if crate::utils::is_jsx_path(uri.path()) {
            return Ok(crate::ide::JsxSemanticTokensService::tokens_range(
                &content,
                uri,
                params.range,
            ));
        }

        Ok(SemanticTokensService::get_tokens_range(
            &content,
            uri,
            params.range,
        ))
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        if !self.state.lsp_features().code_lens {
            return Ok(None);
        }
        Ok(super::annotations::code_lens(&self.state, &params))
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        super::workspace_symbols::search(self, &params).await
    }

    async fn will_rename_files(&self, params: RenameFilesParams) -> Result<Option<WorkspaceEdit>> {
        Ok(super::workspace_files::will_rename_files(&self.state, &params).await)
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        super::workspace_files::did_change_watched_files(self, &params).await;
    }

    async fn did_create_files(&self, params: CreateFilesParams) {
        super::workspace_files::did_create_files(self, &params).await;
    }

    async fn did_delete_files(&self, params: DeleteFilesParams) {
        super::workspace_files::did_delete_files(self, &params).await;
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
        super::workspace_files::did_rename_files(self, &params).await;
    }

    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
        if !self.state.lsp_features().document_links {
            return Ok(None);
        }

        let uri = &params.text_document.uri;

        let Some(content) = self.state.documents.text(uri) else {
            return Ok(None);
        };
        let links = DocumentLinkService::get_links(&content, uri);

        if links.is_empty() {
            Ok(None)
        } else {
            Ok(Some(links))
        }
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        if !self.state.lsp_features().inlay_hints {
            return Ok(None);
        }
        Ok(super::annotations::inlay_hint(&self.state, &params))
    }

    /// Colour swatches for the CSS a `.vue` file authors. Rides the
    /// `document_links` flag: both decorate a literal in the authored text and
    /// make it interactive — a path you can follow, a colour you can pick.
    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        if !self.state.lsp_features().document_links {
            return Ok(Vec::new());
        }
        Ok(super::annotations::document_color(&self.state, &params))
    }

    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        if !self.state.lsp_features().document_links {
            return Ok(Vec::new());
        }
        Ok(super::annotations::color_presentation(&params))
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        if !self.state.lsp_features().folding_ranges {
            return Ok(None);
        }
        Ok(super::document_structure::folding_ranges(
            &self.state,
            &params,
        ))
    }

    /// Expand/shrink selection over the **authored** `.vue` document.
    ///
    /// Shares the `folding_ranges` flag with `folding_range`: both are
    /// document-structure providers built from the same block layout.
    async fn selection_range(
        &self,
        params: SelectionRangeParams,
    ) -> Result<Option<Vec<SelectionRange>>> {
        if !self.state.lsp_features().folding_ranges {
            return Ok(None);
        }
        Ok(super::document_structure::selection_ranges(
            &self.state,
            &params,
        ))
    }

    /// Keep an open/close tag-name pair in sync while the user types.
    ///
    /// Shares the `rename` flag with `rename`/`prepare_rename`: linked editing
    /// is rename-as-you-type over the same authored tag names.
    async fn linked_editing_range(
        &self,
        params: LinkedEditingRangeParams,
    ) -> Result<Option<LinkedEditingRanges>> {
        if !self.state.lsp_features().rename {
            return Ok(None);
        }

        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;
        let Some(content) = self.state.documents.text(uri) else {
            return Ok(None);
        };
        let Some(offset) = position_to_offset(&content, position.line, position.character) else {
            return Ok(None);
        };

        Ok(crate::ide::linked_editing::LinkedEditingService::ranges(
            &content,
            uri.path(),
            offset,
        ))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        if !self.state.lsp_features().formatting {
            return Ok(None);
        }

        let uri = &params.text_document.uri;

        // Standalone (petite-vue) HTML documents are not SFCs: running the SFC
        // formatter over them corrupts the file. Skip until a dedicated HTML
        // formatter lands (#1393).
        if crate::utils::is_standalone_html_path(uri.path()) {
            return Ok(None);
        }

        let Some(_content) = self.state.documents.text(uri) else {
            return Ok(None);
        };
        #[cfg(feature = "glyph")]
        {
            let options = self.state.get_format_options();
            return Ok(super::format::format_document(&_content, &options));
        }
        #[cfg(not(feature = "glyph"))]
        Ok(None)
    }

    /// Format only the SFC blocks the selection touches — see
    /// `server::format::range` for why this is not the whole-document edit.
    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        if !self.state.lsp_features().formatting {
            return Ok(None);
        }

        let uri = &params.text_document.uri;
        let _range = params.range;

        // See `formatting`: standalone HTML must not go through the SFC formatter.
        if crate::utils::is_standalone_html_path(uri.path()) {
            return Ok(None);
        }

        let Some(_content) = self.state.documents.text(uri) else {
            return Ok(None);
        };
        #[cfg(feature = "glyph")]
        {
            let options = self.state.get_format_options();
            let path = uri.path();
            return Ok(super::format::format_range(
                &_content, path, _range, &options,
            ));
        }
        #[cfg(not(feature = "glyph"))]
        Ok(None)
    }

    /// Re-indent the line being typed on — see `server::format::on_type` for
    /// why this never rewrites content.
    async fn on_type_formatting(
        &self,
        params: DocumentOnTypeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        if !self.state.lsp_features().formatting {
            return Ok(None);
        }

        let uri = &params.text_document_position.text_document.uri;

        // See `formatting`: standalone HTML must not go through the SFC formatter.
        if crate::utils::is_standalone_html_path(uri.path()) {
            return Ok(None);
        }

        let Some(_content) = self.state.documents.text(uri) else {
            return Ok(None);
        };
        #[cfg(feature = "glyph")]
        {
            let options = self.state.get_format_options();
            let position = params.text_document_position.position;
            let path = uri.path();
            return Ok(super::format::format_on_type(
                &_content, path, position, &options,
            ));
        }
        #[cfg(not(feature = "glyph"))]
        Ok(None)
    }
}

#[cfg(test)]
mod tests;
