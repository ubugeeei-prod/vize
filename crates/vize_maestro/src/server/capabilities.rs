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

        // Range formatting
        document_range_formatting_provider: features.formatting.then_some(OneOf::Left(true)),

        // Signature help
        signature_help_provider: features.completion.then_some(SignatureHelpOptions {
            trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
            retrigger_characters: None,
            work_done_progress_options: WorkDoneProgressOptions::default(),
        }),

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
            resolve_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        }),

        // Folding ranges
        folding_range_provider: features
            .folding_ranges
            .then_some(FoldingRangeProviderCapability::Simple(true)),

        // Selection ranges
        selection_range_provider: features
            .folding_ranges
            .then_some(SelectionRangeProviderCapability::Simple(true)),

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
