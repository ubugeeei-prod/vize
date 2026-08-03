use super::{LspFeatureConfig, workspace_file_operations};

#[test]
fn workspace_symbols_register_vue_file_events_without_typechecking_or_rename() {
    let mut features = LspFeatureConfig::default();
    features.typecheck = false;
    features.workspace_symbols = true;
    features.file_rename = false;

    let operations = workspace_file_operations(features)
        .expect("workspace symbol lifecycle should be advertised");
    for options in [
        operations.did_create,
        operations.did_delete,
        operations.did_rename,
    ] {
        let options = options.expect("every Vue file event must be registered");
        assert_eq!(options.filters.len(), 1);
        assert_eq!(options.filters[0].pattern.glob, "**/*.vue");
    }
}
