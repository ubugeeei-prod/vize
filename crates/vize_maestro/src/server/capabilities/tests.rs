use super::*;
use crate::ide::trigger_characters;

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
        auto_insert: true,
        cross_file: true,
    }
}

#[test]
fn text_document_sync_uses_incremental_open_close_and_save_without_text() {
    let capabilities = server_capabilities(LspFeatureConfig::default());
    let sync = capabilities
        .text_document_sync
        .expect("text document sync should be advertised");

    let TextDocumentSyncCapability::Options(options) = sync else {
        panic!("text document sync must use explicit options");
    };

    assert_eq!(options.open_close, Some(true));
    assert_eq!(options.change, Some(TextDocumentSyncKind::INCREMENTAL));
    assert_eq!(options.will_save, Some(false));
    assert_eq!(options.will_save_wait_until, Some(false));

    let Some(TextDocumentSyncSaveOptions::SaveOptions(save)) = options.save else {
        panic!("save options should be advertised");
    };
    assert_eq!(save.include_text, Some(false));
}

#[test]
fn completion_triggers_match_completion_service_contract() {
    let capabilities = server_capabilities(LspFeatureConfig::default());
    let advertised = capabilities
        .completion_provider
        .expect("completion should be advertised")
        .trigger_characters
        .expect("completion triggers should be advertised");

    assert_eq!(advertised, trigger_characters());
}

#[test]
fn individual_feature_flags_gate_matching_providers() {
    let cases: &[(
        &str,
        fn(&mut LspFeatureConfig),
        fn(&ServerCapabilities) -> bool,
    )] = &[
        (
            "completion",
            |features| features.completion = false,
            |capabilities| capabilities.completion_provider.is_some(),
        ),
        (
            "hover",
            |features| features.hover = false,
            |capabilities| capabilities.hover_provider.is_some(),
        ),
        (
            "definition",
            |features| features.definition = false,
            |capabilities| capabilities.definition_provider.is_some(),
        ),
        (
            "references",
            |features| features.references = false,
            |capabilities| {
                capabilities.references_provider.is_some()
                    && capabilities.document_highlight_provider.is_some()
            },
        ),
        (
            "document_symbols",
            |features| features.document_symbols = false,
            |capabilities| capabilities.document_symbol_provider.is_some(),
        ),
        (
            "workspace_symbols",
            |features| features.workspace_symbols = false,
            |capabilities| capabilities.workspace_symbol_provider.is_some(),
        ),
        (
            "rename",
            |features| features.rename = false,
            |capabilities| capabilities.rename_provider.is_some(),
        ),
        (
            "formatting",
            |features| features.formatting = false,
            // All three formatting commands ride the same flag: they are one
            // formatter scoped to the document, to a selection, and to a line.
            |capabilities| {
                capabilities.document_formatting_provider.is_some()
                    && capabilities.document_range_formatting_provider.is_some()
                    && capabilities.document_on_type_formatting_provider.is_some()
            },
        ),
        (
            "code_lens",
            |features| features.code_lens = false,
            |capabilities| capabilities.code_lens_provider.is_some(),
        ),
        (
            "semantic_tokens",
            |features| features.semantic_tokens = false,
            |capabilities| capabilities.semantic_tokens_provider.is_some(),
        ),
        (
            "document_links",
            |features| features.document_links = false,
            |capabilities| capabilities.document_link_provider.is_some(),
        ),
        (
            "folding_ranges",
            |features| features.folding_ranges = false,
            |capabilities| capabilities.folding_range_provider.is_some(),
        ),
        (
            "inlay_hints",
            |features| features.inlay_hints = false,
            |capabilities| capabilities.inlay_hint_provider.is_some(),
        ),
    ];

    for (name, disable, provider_is_some) in cases {
        let enabled_capabilities = server_capabilities(all_features());
        assert!(
            provider_is_some(&enabled_capabilities),
            "{name} provider should be advertised before the feature is disabled"
        );

        let mut features = all_features();
        disable(&mut features);
        let disabled_capabilities = server_capabilities(features);
        assert!(
            !provider_is_some(&disabled_capabilities),
            "{name} provider should not be advertised when disabled"
        );
    }
}

#[test]
fn code_actions_require_both_lint_and_code_action_features() {
    let mut features = all_features();
    assert!(server_capabilities(features).code_action_provider.is_some());

    features.lint = false;
    assert!(
        server_capabilities(features).code_action_provider.is_none(),
        "code actions are lint quick fixes and must disappear when lint is off"
    );

    features = all_features();
    features.code_actions = false;
    assert!(
        server_capabilities(features).code_action_provider.is_none(),
        "code action provider must follow the explicit code_actions flag"
    );
}

#[test]
fn declaration_events_remain_registered_when_file_rename_is_disabled() {
    let mut features = all_features();
    let enabled_capabilities = server_capabilities(features);
    let enabled = enabled_capabilities
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.file_operations.as_ref());
    assert!(enabled.is_some());

    features.file_rename = false;
    let capabilities = server_capabilities(features);
    let workspace = capabilities
        .workspace
        .expect("workspace capabilities should still be advertised");

    assert!(workspace.workspace_folders.is_some());
    let operations = workspace
        .file_operations
        .expect("type checking must continue tracking declaration files");
    assert!(operations.did_create.is_some());
    assert!(operations.did_delete.is_some());
    assert!(operations.did_rename.is_some());
    assert!(operations.will_rename.is_none());
}

#[test]
fn linked_editing_shares_the_rename_flag() {
    let capabilities = server_capabilities(all_features());
    assert!(matches!(
        capabilities.linked_editing_range_provider,
        Some(LinkedEditingRangeServerCapabilities::Simple(true))
    ));

    let mut features = all_features();
    features.rename = false;
    let capabilities = server_capabilities(features);
    assert!(capabilities.rename_provider.is_none());
    assert!(capabilities.linked_editing_range_provider.is_none());
    // Only the rename group is affected.
    assert!(capabilities.references_provider.is_some());
    assert!(capabilities.document_symbol_provider.is_some());
}

#[test]
fn selection_ranges_share_the_document_structure_flag_with_folding_ranges() {
    let mut features = all_features();
    features.folding_ranges = false;
    let capabilities = server_capabilities(features);

    assert!(capabilities.folding_range_provider.is_none());
    assert!(capabilities.selection_range_provider.is_none());
    // Only the structure group is affected.
    assert!(capabilities.document_symbol_provider.is_some());
    assert!(capabilities.hover_provider.is_some());
}

#[test]
fn default_features_advertise_non_opinionated_providers() {
    let capabilities = server_capabilities(LspFeatureConfig::default());

    assert!(capabilities.signature_help_provider.is_none());
    assert!(matches!(
        capabilities.selection_range_provider,
        Some(SelectionRangeProviderCapability::Simple(true))
    ));
    assert!(capabilities.completion_provider.is_some());
    assert!(capabilities.hover_provider.is_some());
    assert!(capabilities.definition_provider.is_some());
    assert!(capabilities.document_link_provider.is_some());
    assert!(capabilities.document_formatting_provider.is_none());
    assert!(capabilities.document_range_formatting_provider.is_none());
    assert!(capabilities.document_on_type_formatting_provider.is_none());
}

/// The trigger set is `@vue/language-server`'s, character for character, so an
/// editor configured for one server behaves the same under the other (#3456).
#[test]
fn on_type_formatting_advertises_the_vue_language_server_trigger_set() {
    let options = server_capabilities(all_features())
        .document_on_type_formatting_provider
        .expect("on-type formatting rides the formatting flag");

    assert_eq!(options.first_trigger_character, ";");
    assert_eq!(
        options.more_trigger_character,
        Some(vec!["}".to_string(), "\n".to_string()])
    );
}

#[test]
fn all_features_skip_unimplemented_providers_and_keep_implemented_ones() {
    let capabilities = server_capabilities(all_features());

    assert!(capabilities.signature_help_provider.is_none());
    assert!(matches!(
        capabilities.selection_range_provider,
        Some(SelectionRangeProviderCapability::Simple(true))
    ));
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
    assert!(capabilities.document_range_formatting_provider.is_some());
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

#[test]
fn file_rename_registration_mentions_declaration_files() {
    let options = file_rename_registration_options();
    let file_glob = &options.filters[0].pattern.glob;

    for extension in ["d.ts", "d.mts", "d.cts"] {
        assert!(
            file_glob.contains(extension),
            "rename operations should include declaration shims: {file_glob}"
        );
    }
}

#[test]
fn declaration_file_operations_cover_create_delete_and_rename() {
    let mut features = all_features();
    features.file_rename = false;
    let operations = server_capabilities(features)
        .workspace
        .and_then(|workspace| workspace.file_operations)
        .expect("declaration tracking should be advertised");

    for options in [
        operations.did_create,
        operations.did_delete,
        operations.did_rename,
    ] {
        let options = options.expect("every declaration event must be registered");
        assert_eq!(options.filters.len(), 3);
        assert_eq!(options.filters[0].pattern.glob, "**/*.d.{ts,mts,cts}");
        assert_eq!(options.filters[1].pattern.glob, "**/*.vue");
        assert_eq!(options.filters[2].pattern.glob, "**/*");
        assert!(options.filters[2].pattern.matches == Some(FileOperationPatternKind::Folder));
    }
}
