//! Complete editor vectors for the second default Vue 3 migration slice.

use super::DiagnosticService;
use crate::server::ServerState;
use serde_json::{Value, json};
use tower_lsp::lsp_types::Url;

#[test]
fn editor_default_migrations_preserve_full_vectors_and_legacy_off_controls() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/lint-default-migrations/cases.json"
    ))
    .unwrap();
    let disabled: serde_json::Map<_, _> = corpus["promotedRules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| (name.as_str().unwrap().to_owned(), json!("off")))
        .collect();
    for (config, baseline) in [
        (None, false),
        (Some(json!({"linter":{"preset":"recommended"}})), false),
        (Some(json!({"linter":{"rules":disabled}})), true),
        (Some(json!({"vue":{"version":"2"}})), true),
        (Some(json!({"vue":{"version":"2.7"}})), true),
        (Some(json!({"vue":{"version":"3"}})), false),
        (Some(json!({"linter":{"preset":"incremental"}})), true),
    ] {
        let root = tempfile::tempdir().unwrap();
        if let Some(config) = &config {
            std::fs::write(root.path().join("vize.config.json"), config.to_string()).unwrap();
        }
        let state = ServerState::new();
        state.apply_lsp_initialization_options(Some(&json!({"lint":true,"typecheck":false})));
        state.apply_initialize_workspace_folders(None, Some(root.path()));
        for case in corpus["cases"].as_array().unwrap() {
            let uri =
                Url::from_file_path(root.path().join(case["filename"].as_str().unwrap())).unwrap();
            state.documents.open(
                uri.clone(),
                case["source"].as_str().unwrap().into(),
                1,
                "vue".into(),
            );
            let expected = if config
                .as_ref()
                .is_some_and(|c| c["linter"]["preset"] == "incremental")
            {
                json!([])
            } else if baseline {
                case["baselineLspDiagnostics"].clone()
            } else {
                case["lspDiagnostics"].clone()
            };
            assert_eq!(
                serde_json::to_value(DiagnosticService::collect_lint_only(&state, &uri)).unwrap(),
                expected,
                "{} / {config:?}",
                case["id"]
            );
        }
    }
}
