//! LSP protocol handler implementations.

mod formatting;
mod inlay_hint;
mod linked_editing;
mod properties;
mod typed;

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

use super::MaestroServer;
use crate::ide::position_to_offset;

#[cfg(feature = "native")]
use crate::ide::CompletionService;

mod call_hierarchy;
mod navigation;
mod references;
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
        self.state.record_client_capabilities(&params.capabilities);
        // Resolve workspace root
        let workspace_path = self.state.primary_workspace_path(&params);

        // Set workspace root for native features (Corsa, batch checker)
        #[cfg(feature = "native")]
        if let Some(ref path) = workspace_path {
            tracing::info!("Setting workspace root: {:?}", path);
            self.state.set_workspace_root(path.clone());
        }

        // Load format config from workspace root (always, regardless of feature)
        if let Some(ref path) = workspace_path {
            self.state.load_workspace_config(path);
        }
        // Record every workspace folder so per-document features resolve their own folder's config in multi-root sessions (#3240).
        self.state.apply_initialize_workspace_folders(
            params.workspace_folders.as_deref(),
            workspace_path.as_deref(),
        );
        self.state
            .apply_lsp_initialization_options(params.initialization_options.as_ref());

        Ok(InitializeResult {
            capabilities: self.client_capabilities(),
            server_info: Some(ServerInfo {
                name: "vize-maestro".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
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
        self.finish_shutdown()
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
        let server = self.for_document(&params.text_document.uri).await;
        let uri = params.text_document.uri;
        server.publish_saved_diagnostics(&uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.close_document(params).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let server = self.for_pos(&params.text_document_position_params).await;
        server.native_request(server.hover_request(params)).await
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let server = self.for_pos(&params.text_document_position).await;
        server
            .native_request(server.completion_request(params))
            .await
    }

    async fn completion_resolve(&self, item: CompletionItem) -> Result<CompletionItem> {
        let uri = item
            .data
            .as_ref()
            .and_then(|data| data.get("vizeCompletion"))
            .and_then(|data| data.get("uri"))
            .and_then(serde_json::Value::as_str)
            .and_then(|uri| tower_lsp::lsp_types::Url::parse(uri).ok());
        let server = match &uri {
            Some(uri) => Some(self.for_document(uri).await),
            None => None,
        };
        let server = server.as_deref().unwrap_or(self);
        server
            .native_request(async {
                #[cfg(feature = "native")]
                let item = CompletionService::resolve(&server.state, item).await;
                Ok(item)
            })
            .await
    }

    async fn signature_help(&self, params: SigHelpParams) -> Result<Option<SigHelp>> {
        let server = self.for_pos(&params.text_document_position_params).await;
        server
            .native_request(async { signature_help::signature_help(&server, params).await })
            .await
    }

    async fn goto_definition(&self, params: DefParams) -> Result<Option<DefResponse>> {
        let server = self.for_pos(&params.text_document_position_params).await;
        server
            .native_request(async { navigation::goto_definition(&server, params).await })
            .await
    }

    async fn goto_type_definition(&self, params: TypeDefParams) -> Result<Option<TypeDefResponse>> {
        let server = self.for_pos(&params.text_document_position_params).await;
        server
            .native_request(async { navigation::goto_type_definition(&server, params).await })
            .await
    }

    async fn goto_declaration(&self, params: DeclParams) -> Result<Option<DeclResponse>> {
        let server = self.for_pos(&params.text_document_position_params).await;
        server
            .native_request(async { navigation::goto_declaration(&server, params).await })
            .await
    }

    async fn goto_implementation(&self, params: ImplParams) -> Result<Option<ImplResponse>> {
        let server = self.for_pos(&params.text_document_position_params).await;
        server
            .native_request(async { navigation::goto_implementation(&server, params).await })
            .await
    }

    async fn prepare_call_hierarchy(&self, params: CHPrepareParams) -> Result<Option<CHItems>> {
        let server = self.for_pos(&params.text_document_position_params).await;
        server
            .native_request(async { call_hierarchy::prepare(&server, params).await })
            .await
    }

    async fn incoming_calls(&self, params: CHIncomingParams) -> Result<Option<CHIncomingResponse>> {
        let server = self.for_document(&params.item.uri).await;
        server
            .native_request(async { call_hierarchy::incoming(&server, params).await })
            .await
    }

    async fn outgoing_calls(&self, params: CHOutgoingParams) -> Result<Option<CHOutgoingResponse>> {
        let server = self.for_document(&params.item.uri).await;
        server
            .native_request(async { call_hierarchy::outgoing(&server, params).await })
            .await
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let server = self.for_pos(&params.text_document_position).await;
        server
            .native_request(async { references::references(&server, params).await })
            .await
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        self.document_highlight_request(params).await
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        self.document_symbol_request(params).await
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let server = self.for_document(&params.text_document.uri).await;
        server
            .native_request(async { Ok(super::code_actions::code_actions(&server, &params).await) })
            .await
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let server = self.for_document(&params.text_document.uri).await;
        server
            .native_request(server.prepare_rename_request(params))
            .await
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let server = self.for_pos(&params.text_document_position).await;
        server.native_request(server.rename_request(params)).await
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        self.semantic_tokens_full_request(params).await
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> Result<Option<SemanticTokensRangeResult>> {
        self.semantic_tokens_range_request(params).await
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        self.code_lens_request(params).await
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        super::workspace_symbols::search(self, &params).await
    }

    async fn will_rename_files(&self, params: RenameFilesParams) -> Result<Option<WorkspaceEdit>> {
        super::workspace_files::will_rename_files(self, &params).await
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
        self.document_link_request(params).await
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let server = self.for_document(&params.text_document.uri).await;
        server
            .native_request(server.inlay_hint_request(params))
            .await
    }

    /// Colour swatches for the CSS a `.vue` file authors. Rides the
    /// `document_links` flag: both decorate a literal in the authored text and
    /// make it interactive — a path you can follow, a colour you can pick.
    async fn document_color(&self, params: DocumentColorParams) -> Result<Vec<ColorInformation>> {
        self.document_color_request(params).await
    }

    async fn color_presentation(
        &self,
        params: ColorPresentationParams,
    ) -> Result<Vec<ColorPresentation>> {
        self.color_presentation_request(params).await
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        self.folding_range_request(params).await
    }

    /// Expand/shrink selection over the **authored** `.vue` document.
    ///
    /// Shares the `folding_ranges` flag with `folding_range`: both are
    /// document-structure providers built from the same block layout.
    async fn selection_range(
        &self,
        params: SelectionRangeParams,
    ) -> Result<Option<Vec<SelectionRange>>> {
        self.selection_range_request(params).await
    }

    /// Keep an open/close tag-name pair in sync while the user types.
    ///
    /// Shares the `rename` flag with `rename`/`prepare_rename`: linked editing
    /// is rename-as-you-type over the same authored tag names.
    async fn linked_editing_range(
        &self,
        params: LinkedEditingRangeParams,
    ) -> Result<Option<LinkedEditingRanges>> {
        let server = self.for_pos(&params.text_document_position_params).await;
        linked_editing::linked_editing_range(&server, params).await
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let server = self.for_document(&params.text_document.uri).await;
        formatting::formatting(&server, params).await
    }

    /// Format only the SFC blocks the selection touches — see
    /// `server::format::range` for why this is not the whole-document edit.
    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let server = self.for_document(&params.text_document.uri).await;
        formatting::range_formatting(&server, params).await
    }

    /// Re-indent the line being typed on — see `server::format::on_type` for
    /// why this never rewrites content.
    async fn on_type_formatting(
        &self,
        params: DocumentOnTypeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        let server = self.for_pos(&params.text_document_position).await;
        formatting::on_type_formatting(&server, params).await
    }
}

#[cfg(test)]
#[expect(clippy::disallowed_methods, reason = "fixtures use std String")]
mod tests;
