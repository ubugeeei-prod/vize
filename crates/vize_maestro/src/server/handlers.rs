//! LSP protocol handler implementations.
//!
//! Implements the `LanguageServer` trait for `MaestroServer`, dispatching
//! requests to the appropriate IDE services.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use tower_lsp::{
    LanguageServer,
    jsonrpc::Result,
    lsp_types::{
        CodeActionParams, CodeActionResponse, CodeLens, CodeLensParams, CompletionItem,
        CompletionParams, CompletionResponse, DidChangeConfigurationParams,
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams, DocumentFormattingParams, DocumentHighlight,
        DocumentHighlightParams, DocumentLink, DocumentLinkParams, DocumentRangeFormattingParams,
        DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, FoldingRange,
        FoldingRangeKind, FoldingRangeParams, GotoDefinitionParams, GotoDefinitionResponse, Hover,
        HoverParams, InitializeParams, InitializeResult, InitializedParams, InlayHint,
        InlayHintParams, Location, MessageType, Position, PrepareRenameResponse, Range,
        ReferenceParams, RenameFilesParams, RenameParams, SemanticTokensParams,
        SemanticTokensRangeParams, SemanticTokensRangeResult, SemanticTokensResult, ServerInfo,
        SymbolInformation, SymbolKind, TextDocumentPositionParams, TextEdit, WorkspaceEdit,
        WorkspaceSymbolParams,
    },
};

use super::{MaestroServer, server_capabilities};
use crate::ide::{
    CodeActionService, CodeLensService, CompletionService, DefinitionService,
    DocumentHighlightService, DocumentLinkService, FileRenameService, HoverService, IdeContext,
    InlayHintService, ReferencesService, RenameService, SemanticTokensService,
    WorkspaceSymbolsService, position_to_offset,
};

#[tower_lsp::async_trait]
impl LanguageServer for MaestroServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Resolve workspace root
        let workspace_path = params
            .root_uri
            .as_ref()
            .and_then(|u| u.to_file_path().ok())
            .or_else(|| {
                params
                    .workspace_folders
                    .as_ref()
                    .and_then(|f| f.first())
                    .and_then(|f| f.uri.to_file_path().ok())
            });

        // Load format config from workspace root (always, regardless of feature)
        if let Some(ref path) = workspace_path {
            self.state.load_workspace_config(path);
        }

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
        self.client
            .log_message(MessageType::INFO, "vize_maestro LSP server initialized")
            .await;
    }

    async fn did_change_configuration(&self, _params: DidChangeConfigurationParams) {
        tracing::debug!(
            "Received workspace/didChangeConfiguration; VS Code restarts the server for Vize configuration changes"
        );
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;
        let language_id = params.text_document.language_id;

        self.state
            .documents
            .open(uri.clone(), content.clone(), version, language_id);

        // Generate virtual documents for the SFC
        self.state.update_virtual_docs(&uri, &content);

        self.publish_diagnostics(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        self.state
            .documents
            .apply_changes(&uri, params.content_changes, version);

        // Regenerate virtual documents with updated content
        if let Some(doc) = self.state.documents.get(&uri) {
            let content = doc.text();
            self.state.update_virtual_docs(&uri, &content);
        }

        self.publish_diagnostics(&uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        self.publish_diagnostics(&uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        self.state.documents.close(&uri);

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

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
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

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
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

        {
            #[cfg(feature = "native")]
            {
                let corsa_bridge = self.state.get_corsa_bridge().await;
                if let Some(response) =
                    CompletionService::complete_with_corsa(&ctx, corsa_bridge).await
                {
                    return Ok(Some(response));
                }
            }

            #[cfg(not(feature = "native"))]
            {
                if let Some(response) = CompletionService::complete(&ctx) {
                    return Ok(Some(response));
                }
            }
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

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        if !self.state.lsp_features().definition {
            return Ok(None);
        }

        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
        let Some(offset) = position_to_offset(&content, position.line, position.character) else {
            return Ok(None);
        };

        let ctx = IdeContext::with_content(&self.state, uri, offset, content);

        // Type-aware go-to-definition for `.jsx`/`.tsx` (opt-in
        // `typeChecker.jsxTypecheck`). React `.tsx` is untouched when off.
        #[cfg(feature = "native")]
        if crate::utils::is_jsx_path(uri.path()) {
            if self.state.jsx_typecheck_enabled() {
                let corsa_bridge = self.state.get_corsa_bridge().await;
                if let Some(response) = crate::ide::JsxService::definition(&ctx, corsa_bridge).await
                {
                    return Ok(Some(response));
                }
            }
            return Ok(None);
        }

        {
            #[cfg(feature = "native")]
            {
                let corsa_bridge = self.state.get_corsa_bridge().await;
                if let Some(response) =
                    DefinitionService::definition_with_corsa(&ctx, corsa_bridge).await
                {
                    return Ok(Some(response));
                }
            }

            #[cfg(not(feature = "native"))]
            if let Some(response) = DefinitionService::definition(&ctx) {
                return Ok(Some(response));
            }
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        if !self.state.lsp_features().references {
            return Ok(None);
        }

        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
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

        {
            #[cfg(feature = "native")]
            {
                let corsa_bridge = self.state.get_corsa_bridge().await;
                if let Some(locations) = ReferencesService::references_with_corsa(
                    &ctx,
                    include_declaration,
                    corsa_bridge,
                )
                .await
                {
                    return Ok(Some(locations));
                }
            }

            #[cfg(not(feature = "native"))]
            {
                if let Some(locations) = ReferencesService::references(&ctx, include_declaration) {
                    return Ok(Some(locations));
                }
            }
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

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
        let Some(offset) = position_to_offset(&content, position.line, position.character) else {
            return Ok(None);
        };

        let ctx = IdeContext::with_content(&self.state, uri, offset, content);

        Ok(DocumentHighlightService::highlights(&ctx))
    }

    #[allow(deprecated)]
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        if !self.state.lsp_features().document_symbols {
            return Ok(None);
        }

        let uri = &params.text_document.uri;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();

        // `.jsx`/`.tsx` documents have no SFC blocks; list their component
        // functions instead. Structural (parse-based), so it is not gated on
        // `typeChecker.jsxTypecheck`.
        if crate::utils::is_jsx_path(uri.path()) {
            return Ok(
                crate::ide::JsxDocumentSymbolsService::symbols(&content, uri)
                    .map(DocumentSymbolResponse::Nested),
            );
        }

        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&content, options) else {
            return Ok(None);
        };

        let mut symbols = Vec::new();

        if let Some(ref template) = descriptor.template {
            symbols.push(DocumentSymbol {
                name: "template".to_string(),
                kind: SymbolKind::MODULE,
                tags: None,
                deprecated: None,
                range: Range {
                    start: Position {
                        line: template.loc.start_line.saturating_sub(1) as u32,
                        character: 0,
                    },
                    end: Position {
                        line: template.loc.end_line.saturating_sub(1) as u32,
                        character: 0,
                    },
                },
                selection_range: Range {
                    start: Position {
                        line: template.loc.start_line.saturating_sub(1) as u32,
                        character: 0,
                    },
                    end: Position {
                        line: template.loc.start_line.saturating_sub(1) as u32,
                        character: 10,
                    },
                },
                detail: template.lang.as_ref().map(|l| l.to_string()),
                children: None,
            });
        }

        if let Some(ref script) = descriptor.script {
            symbols.push(DocumentSymbol {
                name: "script".to_string(),
                kind: SymbolKind::MODULE,
                tags: None,
                deprecated: None,
                range: Range {
                    start: Position {
                        line: script.loc.start_line.saturating_sub(1) as u32,
                        character: 0,
                    },
                    end: Position {
                        line: script.loc.end_line.saturating_sub(1) as u32,
                        character: 0,
                    },
                },
                selection_range: Range {
                    start: Position {
                        line: script.loc.start_line.saturating_sub(1) as u32,
                        character: 0,
                    },
                    end: Position {
                        line: script.loc.start_line.saturating_sub(1) as u32,
                        character: 8,
                    },
                },
                detail: script.lang.as_ref().map(|l| l.to_string()),
                children: None,
            });
        }

        if let Some(ref script_setup) = descriptor.script_setup {
            symbols.push(DocumentSymbol {
                name: "script setup".to_string(),
                kind: SymbolKind::MODULE,
                tags: None,
                deprecated: None,
                range: Range {
                    start: Position {
                        line: script_setup.loc.start_line.saturating_sub(1) as u32,
                        character: 0,
                    },
                    end: Position {
                        line: script_setup.loc.end_line.saturating_sub(1) as u32,
                        character: 0,
                    },
                },
                selection_range: Range {
                    start: Position {
                        line: script_setup.loc.start_line.saturating_sub(1) as u32,
                        character: 0,
                    },
                    end: Position {
                        line: script_setup.loc.start_line.saturating_sub(1) as u32,
                        character: 14,
                    },
                },
                detail: script_setup.lang.as_ref().map(|l| l.to_string()),
                children: None,
            });
        }

        for (i, style) in descriptor.styles.iter().enumerate() {
            #[allow(clippy::disallowed_macros)]
            let name = if let Some(ref module) = style.module {
                format!("style module={}", module)
            } else if style.scoped {
                "style scoped".to_string()
            } else {
                format!("style[{}]", i)
            };

            symbols.push(DocumentSymbol {
                name,
                kind: SymbolKind::MODULE,
                tags: None,
                deprecated: None,
                range: Range {
                    start: Position {
                        line: style.loc.start_line.saturating_sub(1) as u32,
                        character: 0,
                    },
                    end: Position {
                        line: style.loc.end_line.saturating_sub(1) as u32,
                        character: 0,
                    },
                },
                selection_range: Range {
                    start: Position {
                        line: style.loc.start_line.saturating_sub(1) as u32,
                        character: 0,
                    },
                    end: Position {
                        line: style.loc.start_line.saturating_sub(1) as u32,
                        character: 7,
                    },
                },
                detail: style.lang.as_ref().map(|l| l.to_string()),
                children: None,
            });
        }

        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let features = self.state.lsp_features();
        if !features.lint || !features.code_actions {
            return Ok(None);
        }

        let uri = &params.text_document.uri;
        let range = params.range;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();

        // `.jsx`/`.tsx`: surface the fixable Patina/JSX-compiler diagnostics as
        // quickfix code actions. Lint-based (parse-only), so not gated on
        // `typeChecker.jsxTypecheck`.
        if crate::utils::is_jsx_path(uri.path()) {
            let actions = crate::ide::JsxCodeActionService::code_actions(&content, uri, range);
            if actions.is_empty() {
                return Ok(None);
            }
            return Ok(Some(actions));
        }

        let Some(offset) = position_to_offset(&content, range.start.line, range.start.character)
        else {
            return Ok(None);
        };

        if let Some(ctx) = IdeContext::new(&self.state, uri, offset) {
            let actions = CodeActionService::code_actions(&ctx, range);
            if !actions.is_empty() {
                return Ok(Some(actions));
            }
        }

        Ok(None)
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

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
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

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
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

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();

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

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();

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

        let uri = &params.text_document.uri;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
        let lenses = CodeLensService::get_lenses(&content, uri);

        if lenses.is_empty() {
            Ok(None)
        } else {
            Ok(Some(lenses))
        }
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        if !self.state.lsp_features().workspace_symbols {
            return Ok(None);
        }

        let query = &params.query;
        let symbols = WorkspaceSymbolsService::search(&self.state, query);

        if symbols.is_empty() {
            Ok(None)
        } else {
            Ok(Some(symbols))
        }
    }

    async fn will_rename_files(&self, params: RenameFilesParams) -> Result<Option<WorkspaceEdit>> {
        if !self.state.lsp_features().file_rename {
            return Ok(None);
        }

        Ok(FileRenameService::will_rename_files(&self.state, &params).await)
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
        if !self.state.lsp_features().file_rename {
            return;
        }

        let renamed = FileRenameService::did_rename_files(&self.state, &params).await;

        for (old_uri, new_uri) in renamed {
            self.client.publish_diagnostics(old_uri, vec![], None).await;
            self.publish_diagnostics(&new_uri).await;
        }
    }

    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
        if !self.state.lsp_features().document_links {
            return Ok(None);
        }

        let uri = &params.text_document.uri;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
        let links = DocumentLinkService::get_links(&content, uri);

        if links.is_empty() {
            Ok(None)
        } else {
            Ok(Some(links))
        }
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let features = self.state.lsp_features();
        if !features.inlay_hints {
            return Ok(None);
        }

        let uri = &params.text_document.uri;
        let range = params.range;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
        let hints =
            InlayHintService::get_hints_with_ecosystem(&content, uri, range, features.ecosystem);

        if hints.is_empty() {
            Ok(None)
        } else {
            Ok(Some(hints))
        }
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        if !self.state.lsp_features().folding_ranges {
            return Ok(None);
        }

        let uri = &params.text_document.uri;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
        let mut ranges = Vec::new();

        let options = vize_atelier_sfc::SfcParseOptions {
            filename: uri.path().to_string().into(),
            ..Default::default()
        };

        if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&content, options) {
            if let Some(ref template) = descriptor.template
                && template.loc.start_line < template.loc.end_line
            {
                ranges.push(FoldingRange {
                    start_line: template.loc.start_line.saturating_sub(1) as u32,
                    start_character: None,
                    end_line: template.loc.end_line.saturating_sub(1) as u32,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: Some("template".to_string()),
                });
            }

            if let Some(ref script) = descriptor.script_setup
                && script.loc.start_line < script.loc.end_line
            {
                ranges.push(FoldingRange {
                    start_line: script.loc.start_line.saturating_sub(1) as u32,
                    start_character: None,
                    end_line: script.loc.end_line.saturating_sub(1) as u32,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: Some("script setup".to_string()),
                });
            }

            if let Some(ref script) = descriptor.script
                && script.loc.start_line < script.loc.end_line
            {
                ranges.push(FoldingRange {
                    start_line: script.loc.start_line.saturating_sub(1) as u32,
                    start_character: None,
                    end_line: script.loc.end_line.saturating_sub(1) as u32,
                    end_character: None,
                    kind: Some(FoldingRangeKind::Region),
                    collapsed_text: Some("script".to_string()),
                });
            }

            for style in &descriptor.styles {
                if style.loc.start_line < style.loc.end_line {
                    ranges.push(FoldingRange {
                        start_line: style.loc.start_line.saturating_sub(1) as u32,
                        start_character: None,
                        end_line: style.loc.end_line.saturating_sub(1) as u32,
                        end_character: None,
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some("style".to_string()),
                    });
                }
            }
        }

        if ranges.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ranges))
        }
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

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let _content = doc.text();
        #[cfg(feature = "glyph")]
        {
            let options = self.state.get_format_options();
            return Ok(super::format::format_document(&_content, &options));
        }
        #[cfg(not(feature = "glyph"))]
        Ok(None)
    }

    async fn range_formatting(
        &self,
        params: DocumentRangeFormattingParams,
    ) -> Result<Option<Vec<TextEdit>>> {
        if !self.state.lsp_features().formatting {
            return Ok(None);
        }

        let uri = &params.text_document.uri;

        // See `formatting`: standalone HTML must not go through the SFC formatter.
        if crate::utils::is_standalone_html_path(uri.path()) {
            return Ok(None);
        }

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let _content = doc.text();
        #[cfg(feature = "glyph")]
        {
            let options = self.state.get_format_options();
            return Ok(super::format::format_document(&_content, &options));
        }
        #[cfg(not(feature = "glyph"))]
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_lsp::{
        LspService,
        lsp_types::{FormattingOptions, TextDocumentIdentifier, Url, WorkDoneProgressParams},
    };

    fn formatting_params(uri: Url) -> DocumentFormattingParams {
        DocumentFormattingParams {
            text_document: TextDocumentIdentifier { uri },
            options: FormattingOptions::default(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        }
    }

    #[test]
    fn formatting_returns_no_edits_for_standalone_html() {
        let (service, _socket) = LspService::new(MaestroServer::new);
        let server = service.inner();
        server
            .state
            .apply_lsp_initialization_options(Some(&serde_json::json!({ "formatting": true })));

        let uri = Url::parse("file:///index.html").unwrap();
        let source = "<!DOCTYPE html>\n<html><body>\n<div   v-scope=\"{ count: 0 }\" >{{ count }}</div>\n</body></html>\n";
        server
            .state
            .documents
            .open(uri.clone(), source.to_string(), 1, "html".to_string());

        let edits =
            futures::executor::block_on(server.formatting(formatting_params(uri.clone()))).unwrap();
        assert!(
            edits.is_none(),
            "the SFC formatter must not touch standalone HTML documents"
        );

        let range_params = DocumentRangeFormattingParams {
            text_document: TextDocumentIdentifier { uri },
            range: Range::new(Position::new(0, 0), Position::new(0, 1)),
            options: FormattingOptions::default(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        };
        let edits = futures::executor::block_on(server.range_formatting(range_params)).unwrap();
        assert!(
            edits.is_none(),
            "range formatting must not touch standalone HTML documents"
        );

        // Guard against the gate over-matching: SFC formatting must still work.
        #[cfg(feature = "glyph")]
        {
            let vue_uri = Url::parse("file:///App.vue").unwrap();
            let vue_source = "<template>\n<div>hello</div>\n</template>\n";
            server.state.documents.open(
                vue_uri.clone(),
                vue_source.to_string(),
                1,
                "vue".to_string(),
            );
            let edits =
                futures::executor::block_on(server.formatting(formatting_params(vue_uri))).unwrap();
            assert!(edits.is_some(), "Vue SFC formatting must keep working");
        }
    }

    // ----------------------------------------------------------------------
    // JSX/TSX LSP routing (#1498). These exercise the request handlers
    // end-to-end for a standalone `.tsx` document: the structural features
    // (document symbols, semantic tokens, code actions, embedded CSS) answer
    // without a Corsa bridge, and the type-aware features (references, rename)
    // stay gated on `typeChecker.jsxTypecheck`.
    // ----------------------------------------------------------------------

    use tower_lsp::lsp_types::{
        CodeActionContext, CodeActionParams, DocumentSymbolParams, PartialResultParams,
        ReferenceContext, ReferenceParams, RenameParams, SemanticTokensParams,
        TextDocumentPositionParams,
    };

    fn open_tsx(server: &MaestroServer, uri: &Url, source: &str) {
        server.state.documents.open(
            uri.clone(),
            source.to_string(),
            1,
            "typescriptreact".to_string(),
        );
        server.state.update_virtual_docs(uri, source);
    }

    fn tsx_server(source: &str, uri: &Url) -> tower_lsp::LspService<MaestroServer> {
        let (service, _socket) = LspService::new(MaestroServer::new);
        let server = service.inner();
        // Enable the structural LSP features (default off in tests).
        server
            .state
            .apply_lsp_initialization_options(Some(&serde_json::json!({
                "documentSymbols": true,
                "semanticTokens": true,
                "codeActions": true,
                "lint": true,
                "references": true,
                "rename": true,
            })));
        open_tsx(server, uri, source);
        service
    }

    #[test]
    fn document_symbol_lists_tsx_components() {
        let uri = Url::parse("file:///Counter.tsx").unwrap();
        let source = "const Counter = (props: { n: number }) => <span>{props.n}</span>;\n";
        let service = tsx_server(source, &uri);
        let server = service.inner();

        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };
        let response = futures::executor::block_on(server.document_symbol(params)).unwrap();
        match response {
            Some(DocumentSymbolResponse::Nested(symbols)) => {
                assert_eq!(symbols.len(), 1);
                assert_eq!(symbols[0].name, "Counter");
            }
            other => panic!("expected nested TSX component symbols, got: {other:?}"),
        }
    }

    #[test]
    fn semantic_tokens_full_highlights_tsx_expressions() {
        let uri = Url::parse("file:///Comp.tsx").unwrap();
        let source = "const C = (props: { msg: string }) => <div>{props.msg}</div>;\n";
        let service = tsx_server(source, &uri);
        let server = service.inner();

        let params = SemanticTokensParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };
        let response = futures::executor::block_on(server.semantic_tokens_full(params)).unwrap();
        match response {
            Some(SemanticTokensResult::Tokens(tokens)) => {
                assert!(!tokens.data.is_empty(), "expected highlighted TSX tokens");
            }
            other => panic!("expected TSX semantic tokens, got: {other:?}"),
        }
    }

    #[test]
    fn code_action_surfaces_tsx_quickfix() {
        let uri = Url::parse("file:///Comp.tsx").unwrap();
        // Multi-space inside the opening tag is a fixable JSX lint diagnostic.
        let source = "const C = () => <div    class=\"a\">x</div>;\n";
        let service = tsx_server(source, &uri);
        let server = service.inner();

        let params = CodeActionParams {
            text_document: TextDocumentIdentifier { uri },
            range: Range::new(Position::new(0, 0), Position::new(1, 0)),
            context: CodeActionContext::default(),
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };
        let response = futures::executor::block_on(server.code_action(params)).unwrap();
        let actions = response.expect("expected TSX quickfix code actions");
        assert!(
            actions.iter().any(|action| matches!(
                action,
                tower_lsp::lsp_types::CodeActionOrCommand::CodeAction(_)
            )),
            "expected at least one quickfix, got: {actions:?}"
        );
    }

    #[test]
    fn embedded_scoped_style_produces_css_virtual_document() {
        let uri = Url::parse("file:///Styled.tsx").unwrap();
        let source = "const C = () => (\n  <>\n    <div class=\"box\">hi</div>\n    <style scoped>{`\n      .box { color: red; }\n    `}</style>\n  </>\n);\n";
        let service = tsx_server(source, &uri);
        let server = service.inner();

        let docs = server
            .state
            .get_virtual_docs(&uri)
            .expect("virtual docs cached for a TSX with <style scoped>");
        assert_eq!(docs.styles.len(), 1, "one CSS virtual document expected");
        let style = &docs.styles[0];
        assert!(style.uri.as_str().ends_with(".css"));
        assert!(style.content.contains(".box"));
    }

    #[test]
    fn references_gated_off_returns_none_for_tsx() {
        // jsxTypecheck defaults off, so the type-aware references path must not
        // run for `.tsx` (and the SFC fallback never fires for non-SFC JSX).
        let uri = Url::parse("file:///Comp.tsx").unwrap();
        let source = "const C = (props: { msg: string }) => {\n  const total = props.msg;\n  return <span>{total}</span>;\n};\n";
        let service = tsx_server(source, &uri);
        let server = service.inner();
        assert!(
            !server.state.jsx_typecheck_enabled(),
            "precondition: jsxTypecheck is off by default"
        );

        let offset = source.find("total").unwrap();
        let (line, character) = crate::ide::offset_to_position(source, offset);
        let params = ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position::new(line, character),
            },
            context: ReferenceContext {
                include_declaration: true,
            },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };
        let response = futures::executor::block_on(server.references(params)).unwrap();
        assert!(
            response.is_none(),
            "references must be gated off for TSX when jsxTypecheck is disabled"
        );
    }

    #[test]
    fn rename_gated_off_returns_none_for_tsx() {
        let uri = Url::parse("file:///Comp.tsx").unwrap();
        let source = "const C = (props: { msg: string }) => <div>{props.msg}</div>;\n";
        let service = tsx_server(source, &uri);
        let server = service.inner();

        let offset = source.find("msg").unwrap();
        let (line, character) = crate::ide::offset_to_position(source, offset);
        let params = RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position::new(line, character),
            },
            new_name: "renamed".to_string(),
            work_done_progress_params: WorkDoneProgressParams::default(),
        };
        let response = futures::executor::block_on(server.rename(params)).unwrap();
        assert!(
            response.is_none(),
            "rename must be gated off for TSX when jsxTypecheck is disabled"
        );
    }

    #[test]
    fn document_symbol_still_parses_sfc_after_jsx_routing() {
        // Guard against the JSX gate over-matching: a regular `.vue` SFC must
        // still go through the SFC document-symbol path.
        let (service, _socket) = LspService::new(MaestroServer::new);
        let server = service.inner();
        server.state.apply_lsp_initialization_options(Some(
            &serde_json::json!({ "documentSymbols": true }),
        ));
        let uri = Url::parse("file:///App.vue").unwrap();
        let source = "<template>\n  <div>hi</div>\n</template>\n<script setup lang=\"ts\">\nconst x = 1\n</script>\n";
        server
            .state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());

        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: WorkDoneProgressParams::default(),
            partial_result_params: PartialResultParams::default(),
        };
        let response = futures::executor::block_on(server.document_symbol(params)).unwrap();
        match response {
            Some(DocumentSymbolResponse::Nested(symbols)) => {
                assert!(
                    symbols.iter().any(|s| s.name == "template"),
                    "SFC document symbols must still include the template block"
                );
            }
            other => panic!("expected SFC block symbols, got: {other:?}"),
        }
    }
}
