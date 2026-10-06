//! Whole authored LSP lint vectors for implicit and explicit presets.

use super::DiagnosticService;
use crate::server::ServerState;
use serde_json::{Value, json};
use tower_lsp::lsp_types::Url;

fn corpus() -> Value {
    serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/lint-default-correctness/cases.json"
    ))
    .unwrap()
}

#[test]
fn configless_ecosystem_and_recommended_editor_lint_share_complete_defaults() {
    let data = corpus();
    for explicit in [false, true] {
        let root = tempfile::tempdir().unwrap();
        if explicit {
            std::fs::write(
                root.path().join("vize.config.json"),
                r#"{"linter":{"preset":"recommended"}}"#,
            )
            .unwrap();
        }
        let state = ServerState::new();
        state.apply_lsp_initialization_options(Some(&json!({"lint":true,"typecheck":false})));
        state.apply_initialize_workspace_folders(None, Some(root.path()));
        for case in data["cases"].as_array().unwrap() {
            if case["kind"] == "html" {
                continue;
            }
            let uri =
                Url::from_file_path(root.path().join(case["filename"].as_str().unwrap())).unwrap();
            state.documents.open(
                uri.clone(),
                case["source"].as_str().unwrap().into(),
                1,
                "vue".into(),
            );
            let diagnostics = DiagnosticService::collect_lint_only(&state, &uri);
            assert_eq!(
                serde_json::to_value(&diagnostics).unwrap(),
                case["lspDiagnostics"],
                "{} / explicit={explicit}",
                case["id"]
            );
        }
    }
}

#[test]
fn editor_explicit_off_preserves_the_complete_existing_baseline() {
    let data = corpus();
    let root = tempfile::tempdir().unwrap();
    let rules: serde_json::Map<_, _> = data["promotedRules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| (name.as_str().unwrap().to_owned(), json!("off")))
        .collect();
    std::fs::write(
        root.path().join("vize.config.json"),
        json!({"linter":{"rules":rules}}).to_string(),
    )
    .unwrap();
    let state = ServerState::new();
    state.apply_lsp_initialization_options(Some(&json!({"lint":true,"typecheck":false})));
    state.apply_initialize_workspace_folders(None, Some(root.path()));
    for case in &data["cases"].as_array().unwrap()[..6] {
        let uri =
            Url::from_file_path(root.path().join(case["filename"].as_str().unwrap())).unwrap();
        state.documents.open(
            uri.clone(),
            case["source"].as_str().unwrap().into(),
            1,
            "vue".into(),
        );
        assert_eq!(
            serde_json::to_value(DiagnosticService::collect_lint_only(&state, &uri)).unwrap(),
            case["baselineLspDiagnostics"],
            "{}",
            case["id"]
        );
    }
}
