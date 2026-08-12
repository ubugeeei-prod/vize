use super::{LspFeatureConfig, workspace_file_operations};

#[test]
fn workspace_symbols_register_vue_file_events_without_typechecking_or_rename() {
    let features = LspFeatureConfig {
        typecheck: false,
        workspace_symbols: true,
        file_rename: false,
        ..LspFeatureConfig::default()
    };

    let operations = workspace_file_operations(features)
        .expect("workspace symbol lifecycle should be advertised");
    for options in [
        operations.did_create,
        operations.did_delete,
        operations.did_rename,
    ] {
        let options = options.expect("every Vue file event must be registered");
        assert_eq!(options.filters.len(), 2);
        assert_eq!(options.filters[0].pattern.glob, "**/*.vue");
        assert_eq!(options.filters[1].pattern.glob, "**/*");
        assert_eq!(
            options.filters[1].pattern.matches,
            Some(tower_lsp::lsp_types::FileOperationPatternKind::Folder)
        );
    }
}

#[test]
fn typechecking_registers_directory_events_without_workspace_symbols() {
    let features = LspFeatureConfig {
        typecheck: true,
        workspace_symbols: false,
        file_rename: false,
        ..LspFeatureConfig::default()
    };

    let operations = workspace_file_operations(features).expect("typechecking should be tracked");
    for options in [
        operations.did_create,
        operations.did_delete,
        operations.did_rename,
    ] {
        let filters = options.expect("file event registration").filters;
        assert_eq!(filters.len(), 2);
        assert_eq!(filters[0].pattern.glob, "**/*.d.{ts,mts,cts}");
        assert_eq!(filters[1].pattern.glob, "**/*");
        assert_eq!(
            filters[1].pattern.matches,
            Some(tower_lsp::lsp_types::FileOperationPatternKind::Folder)
        );
    }
}
