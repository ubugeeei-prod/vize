use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

const APP: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/",
    "native-inline-event-handler-9835/App.vue.txt"
));
const TSCONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/",
    "native-inline-event-handler-9835/tsconfig.json.txt"
));

#[test]
fn native_inline_event_handler_is_contextually_typed() {
    let project_root = create_project_case(
        "native-inline-event-handler-9835",
        &[("src/App.vue", APP), ("tsconfig.json", TSCONFIG)],
    );

    if resolve_test_tsgo_binary().is_some() {
        assert_eq!(
            snapshot_project_diagnostics(&project_root),
            Some(Vec::new())
        );
    }

    let _ = std::fs::remove_dir_all(&project_root);
}
