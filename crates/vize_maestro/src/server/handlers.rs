//! LSP protocol handler implementations.
//!
//! Implements the `LanguageServer` trait for `MaestroServer`, dispatching
//! requests to the appropriate IDE services.
#![allow(clippy::disallowed_types, clippy::disallowed_methods)]

use tower_lsp::{
    jsonrpc::Result,
    lsp_types::{
        CodeActionParams, CodeActionResponse, CodeLens, CodeLensParams, CompletionItem,
        CompletionParams, CompletionResponse, DidChangeTextDocumentParams,
        DidCloseTextDocumentParams, DidOpenTextDocumentParams, DidSaveTextDocumentParams,
        DocumentFormattingParams, DocumentLink, DocumentLinkParams, DocumentRangeFormattingParams,
        DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, FoldingRange,
        FoldingRangeKind, FoldingRangeParams, GotoDefinitionParams, GotoDefinitionResponse, Hover,
        HoverParams, InitializeParams, InitializeResult, InitializedParams, InlayHint,
        InlayHintParams, Location, MessageType, Position, PrepareRenameResponse, Range,
        ReferenceParams, RenameFilesParams, RenameParams, SemanticTokensParams,
        SemanticTokensResult, ServerInfo, SymbolInformation, SymbolKind,
        TextDocumentPositionParams, TextEdit, WorkspaceEdit, WorkspaceSymbolParams,
    },
    LanguageServer,
};

use super::{server_capabilities, MaestroServer};
use crate::ide::{
    CodeActionService, CodeLensService, CompletionService, DefinitionService, DocumentLinkService,
    FileRenameService, HoverService, IdeContext, InlayHintService, ReferencesService,
    RenameService, SemanticTokensService, WorkspaceSymbolsService,
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
            self.state.load_format_config(path);
        }

        // Set workspace root for native features (Corsa, batch checker)
        #[cfg(feature = "native")]
        if let Some(path) = workspace_path {
            tracing::info!("Setting workspace root: {:?}", path);
            self.state.set_workspace_root(path);
        }

        Ok(InitializeResult {
            capabilities: server_capabilities(),
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
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();

        let offset =
            crate::utils::position_to_offset_str(&content, position.line, position.character);

        let mut hover_result: Option<Hover> = None;

        if let Some(ctx) = IdeContext::new(&self.state, uri, offset) {
            #[cfg(feature = "native")]
            {
                let corsa_bridge = self.state.get_corsa_bridge().await;
                hover_result = HoverService::hover_with_corsa(&ctx, corsa_bridge).await;
            }

            #[cfg(not(feature = "native"))]
            {
                hover_result = HoverService::hover(&ctx);
            }
        }

        let lint_hover = self.get_lint_hover_at_position(uri, &content, position);
        if let Some(lint_info) = lint_hover {
            hover_result = Some(Self::merge_hover_with_lint(hover_result, lint_info));
        }

        Ok(hover_result)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
        let offset =
            crate::utils::position_to_offset_str(&content, position.line, position.character);

        if let Some(ctx) = IdeContext::new(&self.state, uri, offset) {
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
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
        let offset =
            crate::utils::position_to_offset_str(&content, position.line, position.character);

        if let Some(ctx) = IdeContext::new(&self.state, uri, offset) {
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
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let include_declaration = params.context.include_declaration;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
        let offset =
            crate::utils::position_to_offset_str(&content, position.line, position.character);

        if let Some(ctx) = IdeContext::new(&self.state, uri, offset) {
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

    #[allow(deprecated)]
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
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
        let uri = &params.text_document.uri;
        let range = params.range;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
        let offset =
            crate::utils::position_to_offset_str(&content, range.start.line, range.start.character);

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
        let uri = &params.text_document.uri;
        let position = params.position;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
        let offset =
            crate::utils::position_to_offset_str(&content, position.line, position.character);

        if let Some(ctx) = IdeContext::new(&self.state, uri, offset) {
            #[cfg(feature = "native")]
            {
                let corsa_bridge = self.state.get_corsa_bridge().await;
                return Ok(RenameService::prepare_rename_with_corsa(&ctx, corsa_bridge).await);
            }

            #[cfg(not(feature = "native"))]
            {
                return Ok(RenameService::prepare_rename(&ctx));
            }
        }

        Ok(None)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;
        let new_name = &params.new_name;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
        let offset =
            crate::utils::position_to_offset_str(&content, position.line, position.character);

        if let Some(ctx) = IdeContext::new(&self.state, uri, offset) {
            #[cfg(feature = "native")]
            {
                let corsa_bridge = self.state.get_corsa_bridge().await;
                return Ok(RenameService::rename_with_corsa(&ctx, new_name, corsa_bridge).await);
            }

            #[cfg(not(feature = "native"))]
            {
                return Ok(RenameService::rename(&ctx, new_name));
            }
        }

        Ok(None)
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = &params.text_document.uri;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
        Ok(SemanticTokensService::get_tokens(&content, uri))
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
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
        let query = &params.query;
        let symbols = WorkspaceSymbolsService::search(&self.state, query);

        if symbols.is_empty() {
            Ok(None)
        } else {
            Ok(Some(symbols))
        }
    }

    async fn will_rename_files(&self, params: RenameFilesParams) -> Result<Option<WorkspaceEdit>> {
        Ok(FileRenameService::will_rename_files(&self.state, &params).await)
    }

    async fn did_rename_files(&self, params: RenameFilesParams) {
        let renamed = FileRenameService::did_rename_files(&self.state, &params).await;

        for (old_uri, new_uri) in renamed {
            self.client.publish_diagnostics(old_uri, vec![], None).await;
            self.publish_diagnostics(&new_uri).await;
        }
    }

    async fn document_link(&self, params: DocumentLinkParams) -> Result<Option<Vec<DocumentLink>>> {
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
        let uri = &params.text_document.uri;
        let range = params.range;

        let Some(doc) = self.state.documents.get(uri) else {
            return Ok(None);
        };

        let content = doc.text();
        let hints = InlayHintService::get_hints(&content, uri, range);

        if hints.is_empty() {
            Ok(None)
        } else {
            Ok(Some(hints))
        }
    }

    async fn folding_range(&self, params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
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
            if let Some(ref template) = descriptor.template {
                if template.loc.start_line < template.loc.end_line {
                    ranges.push(FoldingRange {
                        start_line: template.loc.start_line.saturating_sub(1) as u32,
                        start_character: None,
                        end_line: template.loc.end_line.saturating_sub(1) as u32,
                        end_character: None,
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some("template".to_string()),
                    });
                }
            }

            if let Some(ref script) = descriptor.script_setup {
                if script.loc.start_line < script.loc.end_line {
                    ranges.push(FoldingRange {
                        start_line: script.loc.start_line.saturating_sub(1) as u32,
                        start_character: None,
                        end_line: script.loc.end_line.saturating_sub(1) as u32,
                        end_character: None,
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some("script setup".to_string()),
                    });
                }
            }

            if let Some(ref script) = descriptor.script {
                if script.loc.start_line < script.loc.end_line {
                    ranges.push(FoldingRange {
                        start_line: script.loc.start_line.saturating_sub(1) as u32,
                        start_character: None,
                        end_line: script.loc.end_line.saturating_sub(1) as u32,
                        end_character: None,
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: Some("script".to_string()),
                    });
                }
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
        let uri = &params.text_document.uri;

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
        let uri = &params.text_document.uri;

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
