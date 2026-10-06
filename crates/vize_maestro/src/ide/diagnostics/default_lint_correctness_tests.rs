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

#[test]
fn configured_editor_vue_versions_preserve_whole_existing_baselines() {
    let data = corpus();
    for version in data["configuredVueVersions"].as_array().unwrap() {
        let version = version.as_str().unwrap();
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join("vize.config.json"),
            json!({"vue":{"version":version}}).to_string(),
        )
        .unwrap();
        let state = ServerState::new();
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
            let expected = if version == "3" {
                &case["lspDiagnostics"]
            } else {
                &case["baselineLspDiagnostics"]
            };
            assert_eq!(
                serde_json::to_value(DiagnosticService::collect_lint_only(&state, &uri)).unwrap(),
                *expected,
                "{} / {version}",
                case["id"]
            );
        }
    }
}

#[test]
fn editor_vue_version_is_owned_by_the_matching_folder_and_global_fallback() {
    use tower_lsp::lsp_types::WorkspaceFolder;
    let data = corpus();
    let temp = tempfile::tempdir().unwrap();
    let roots: Vec<_> = ["legacy", "modern", "configless", "outside"]
        .iter()
        .map(|name| temp.path().join(name))
        .collect();
    for root in &roots {
        std::fs::create_dir(root).unwrap();
    }
    std::fs::write(
        roots[0].join("vize.config.json"),
        r#"{"vue":{"version":"2.7"}}"#,
    )
    .unwrap();
    std::fs::write(
        roots[1].join("vize.config.json"),
        r#"{"vue":{"version":"3"}}"#,
    )
    .unwrap();
    let folders: Vec<_> = roots[..3]
        .iter()
        .enumerate()
        .map(|(index, root)| WorkspaceFolder {
            uri: Url::from_file_path(root).unwrap(),
            name: format!("folder-{index}"),
        })
        .collect();
    let state = ServerState::new();
    state.apply_initialize_workspace_folders(Some(&folders), Some(&roots[0]));
    for (index, root) in roots.iter().enumerate() {
        for case in &data["cases"].as_array().unwrap()[..6] {
            let uri = Url::from_file_path(root.join(case["filename"].as_str().unwrap())).unwrap();
            state.documents.open(
                uri.clone(),
                case["source"].as_str().unwrap().into(),
                1,
                "vue".into(),
            );
            let expected = if index == 0 || index == 3 {
                &case["baselineLspDiagnostics"]
            } else {
                &case["lspDiagnostics"]
            };
            assert_eq!(
                serde_json::to_value(DiagnosticService::collect_lint_only(&state, &uri)).unwrap(),
                *expected,
                "{} / folder={index}",
                case["id"]
            );
        }
    }
}
