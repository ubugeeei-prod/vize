//! LSP server capabilities declaration.
#![allow(clippy::disallowed_methods)]

use tower_lsp::lsp_types::*;

use super::state::LspFeatureConfig;

/// Build the server capabilities to advertise to the client.
pub fn server_capabilities(features: LspFeatureConfig) -> ServerCapabilities {
    ServerCapabilities {
        // Document synchronization
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::INCREMENTAL),
                will_save: Some(false),
                will_save_wait_until: Some(false),
                save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                    include_text: Some(false),
                })),
            },
        )),

        // Hover support
        hover_provider: features
            .hover
            .then_some(HoverProviderCapability::Simple(true)),

        // Completion support
        completion_provider: features.completion.then_some(CompletionOptions {
            trigger_characters: Some(vec![
                ".".to_string(),
                ":".to_string(),
                "@".to_string(),
                "#".to_string(),
                "<".to_string(),
                "/".to_string(),
                "\"".to_string(),
                "'".to_string(),
                " ".to_string(),
            ]),
            resolve_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
            all_commit_characters: None,
            completion_item: None,
        }),

        // Go to definition
        definition_provider: features.definition.then_some(OneOf::Left(true)),

        // Find references
        references_provider: features.references.then_some(OneOf::Left(true)),

        // Highlight matching tags and symbol occurrences in the current document.
        document_highlight_provider: features.references.then_some(OneOf::Left(true)),

        // Document symbols (outline)
        document_symbol_provider: features.document_symbols.then_some(OneOf::Left(true)),

        // Workspace symbols
        workspace_symbol_provider: features.workspace_symbols.then_some(OneOf::Left(true)),

        // Code actions (quick fixes, refactoring)
        code_action_provider: (features.lint && features.code_actions).then_some(
            CodeActionProviderCapability::Options(CodeActionOptions {
                code_action_kinds: Some(vec![
                    CodeActionKind::QUICKFIX,
                    CodeActionKind::REFACTOR,
                    CodeActionKind::SOURCE,
                ]),
                work_done_progress_options: WorkDoneProgressOptions::default(),
                resolve_provider: Some(false),
            }),
        ),

        // Rename support
        rename_provider: features.rename.then_some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        })),

        // Document formatting
        document_formatting_provider: features.formatting.then_some(OneOf::Left(true)),

        // Range formatting is intentionally not advertised until the handler
        // can produce edits scoped to the requested range.
        document_range_formatting_provider: None,

        // Signature help is not implemented yet.
        signature_help_provider: None,

        // Code lens
        code_lens_provider: features.code_lens.then_some(CodeLensOptions {
            resolve_provider: Some(false),
        }),

        // Semantic tokens (syntax highlighting)
        semantic_tokens_provider: features.semantic_tokens.then_some(
            SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
                work_done_progress_options: WorkDoneProgressOptions::default(),
                legend: SemanticTokensLegend {
                    token_types: vec![
                        SemanticTokenType::NAMESPACE,
                        SemanticTokenType::TYPE,
                        SemanticTokenType::CLASS,
                        SemanticTokenType::ENUM,
                        SemanticTokenType::INTERFACE,
                        SemanticTokenType::STRUCT,
                        SemanticTokenType::TYPE_PARAMETER,
                        SemanticTokenType::PARAMETER,
                        SemanticTokenType::VARIABLE,
                        SemanticTokenType::PROPERTY,
                        SemanticTokenType::ENUM_MEMBER,
                        SemanticTokenType::EVENT,
                        SemanticTokenType::FUNCTION,
                        SemanticTokenType::METHOD,
                        SemanticTokenType::MACRO,
                        SemanticTokenType::KEYWORD,
                        SemanticTokenType::MODIFIER,
                        SemanticTokenType::COMMENT,
                        SemanticTokenType::STRING,
                        SemanticTokenType::NUMBER,
                        SemanticTokenType::REGEXP,
                        SemanticTokenType::OPERATOR,
                        SemanticTokenType::DECORATOR,
                    ],
                    token_modifiers: vec![
                        SemanticTokenModifier::DECLARATION,
                        SemanticTokenModifier::DEFINITION,
                        SemanticTokenModifier::READONLY,
                        SemanticTokenModifier::STATIC,
                        SemanticTokenModifier::DEPRECATED,
                        SemanticTokenModifier::ABSTRACT,
                        SemanticTokenModifier::ASYNC,
                        SemanticTokenModifier::MODIFICATION,
                        SemanticTokenModifier::DOCUMENTATION,
                        SemanticTokenModifier::DEFAULT_LIBRARY,
                    ],
                },
                range: Some(true),
                full: Some(SemanticTokensFullOptions::Bool(true)),
            }),
        ),

        // Document links
        document_link_provider: features.document_links.then_some(DocumentLinkOptions {
            resolve_provider: Some(false),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        }),

        // Folding ranges
        folding_range_provider: features
            .folding_ranges
            .then_some(FoldingRangeProviderCapability::Simple(true)),

        // Selection ranges are not implemented yet.
        selection_range_provider: None,

        // Inlay hints
        inlay_hint_provider: features.inlay_hints.then_some(OneOf::Left(true)),

        // Workspace capabilities
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(true)),
            }),
            file_operations: features.file_rename.then_some(
                WorkspaceFileOperationsServerCapabilities {
                    did_create: None,
                    will_create: None,
                    did_rename: Some(file_rename_registration_options()),
                    will_rename: Some(file_rename_registration_options()),
                    did_delete: None,
                    will_delete: None,
                },
            ),
        }),

        // Features not yet implemented
        type_definition_provider: None,
        implementation_provider: None,
        declaration_provider: None,
        color_provider: None,
        document_on_type_formatting_provider: None,
        execute_command_provider: None,
        linked_editing_range_provider: None,
        call_hierarchy_provider: None,
        moniker_provider: None,
        experimental: None,

        // Default for other fields
        ..Default::default()
    }
}

fn file_rename_registration_options() -> FileOperationRegistrationOptions {
    FileOperationRegistrationOptions {
        filters: vec![
            FileOperationFilter {
                scheme: Some("file".to_string()),
                pattern: FileOperationPattern {
                    glob: "**/*.{vue,ts,tsx,js,jsx,mts,cts,mjs,cjs}".to_string(),
                    matches: Some(FileOperationPatternKind::File),
                    options: None,
                },
            },
            FileOperationFilter {
                scheme: Some("file".to_string()),
                pattern: FileOperationPattern {
                    glob: "**/*".to_string(),
                    matches: Some(FileOperationPatternKind::Folder),
                    options: None,
                },
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_features() -> LspFeatureConfig {
        LspFeatureConfig {
            lint: true,
            typecheck: true,
            ecosystem: true,
            options_api: true,
            legacy_vue2: true,
            completion: true,
            hover: true,
            definition: true,
            references: true,
            document_symbols: true,
            workspace_symbols: true,
            code_actions: true,
            rename: true,
            formatting: true,
            code_lens: true,
            semantic_tokens: true,
            document_links: true,
            folding_ranges: true,
            inlay_hints: true,
            file_rename: true,
            cross_file: true,
        }
    }

    #[test]
    fn default_features_advertise_non_opinionated_providers() {
        let capabilities = server_capabilities(LspFeatureConfig::default());

        assert!(capabilities.signature_help_provider.is_none());
        assert!(capabilities.selection_range_provider.is_none());
        assert!(capabilities.completion_provider.is_some());
        assert!(capabilities.hover_provider.is_some());
        assert!(capabilities.definition_provider.is_some());
        assert!(capabilities.document_link_provider.is_some());
        assert!(capabilities.document_formatting_provider.is_none());
    }

    #[test]
    fn all_features_skip_unimplemented_providers_and_keep_implemented_ones() {
        let capabilities = server_capabilities(all_features());

        assert!(capabilities.signature_help_provider.is_none());
        assert!(capabilities.selection_range_provider.is_none());
        assert_eq!(
            capabilities
                .document_link_provider
                .as_ref()
                .and_then(|provider| provider.resolve_provider),
            Some(false)
        );

        assert!(capabilities.completion_provider.is_some());
        assert!(capabilities.hover_provider.is_some());
        assert!(capabilities.definition_provider.is_some());
        assert!(capabilities.references_provider.is_some());
        assert!(capabilities.document_symbol_provider.is_some());
        assert!(capabilities.workspace_symbol_provider.is_some());
        assert!(capabilities.code_action_provider.is_some());
        assert!(capabilities.rename_provider.is_some());
        assert!(capabilities.document_formatting_provider.is_some());
        assert!(capabilities.document_range_formatting_provider.is_none());
        assert!(capabilities.code_lens_provider.is_some());
        assert!(capabilities.semantic_tokens_provider.is_some());
        assert!(capabilities.document_link_provider.is_some());
        assert!(capabilities.folding_range_provider.is_some());
        assert!(capabilities.inlay_hint_provider.is_some());
        assert!(
            capabilities
                .workspace
                .and_then(|workspace| workspace.file_operations)
                .is_some()
        );
    }
}
