use super::*;

/// One gating case: the flag's name, the mutation that disables it, and the
/// capability probe that must go quiet once it is disabled.
type FeatureGatingCase = (
    &'static str,
    fn(&mut LspFeatureConfig),
    fn(&ServerCapabilities) -> bool,
);

#[test]
fn individual_feature_flags_gate_matching_providers() {
    let cases: &[FeatureGatingCase] = &[
        (
            "completion",
            |features| features.completion = false,
            |capabilities| capabilities.completion_provider.is_some(),
        ),
        (
            "signature_help",
            |features| features.signature_help = false,
            |capabilities| capabilities.signature_help_provider.is_some(),
        ),
        (
            "hover",
            |features| features.hover = false,
            |capabilities| capabilities.hover_provider.is_some(),
        ),
        (
            "definition",
            |features| features.definition = false,
            |capabilities| {
                capabilities.definition_provider.is_some()
                    || capabilities.type_definition_provider.is_some()
                    || capabilities.implementation_provider.is_some()
            },
        ),
        (
            "typecheck",
            |features| features.typecheck = false,
            |capabilities| capabilities.implementation_provider.is_some(),
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
