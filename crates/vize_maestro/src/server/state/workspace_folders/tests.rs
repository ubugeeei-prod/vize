use tower_lsp::lsp_types::{Url, WorkspaceFolder, WorkspaceFoldersChangeEvent};
use vize_l0::{config::LintRuleSeverity, cstr};

use crate::server::ServerState;

#[test]
fn scoped_rules_and_global_ignores_match_document_paths() {
    let parent = tempfile::tempdir().unwrap();
    let root = folder_with_config(
        parent.path(),
        "project",
        r#"{
  "entries": [{"files":["src/legacy/**/*.vue"], "ignores":["src/legacy/skip.vue"], "linter":{"rules":{"vue/permitted-contents":"off"}}}],
  "ignores": ["src/generated/**"]
}"#,
    );
    let state = ServerState::new();
    state.apply_initialize_workspace_folders(None, Some(&root));

    let legacy = Url::from_file_path(root.join("src/legacy/table.vue")).unwrap();
    let (legacy_config, _) = state.linter_settings_for_uri(&legacy).unwrap();
    assert_eq!(
        legacy_config.rules.get("vue/permitted-contents"),
        Some(&LintRuleSeverity::Off)
    );

    let other = Url::from_file_path(root.join("src/other.vue")).unwrap();
    let (other_config, _) = state.linter_settings_for_uri(&other).unwrap();
    assert_eq!(other_config.rules.get("vue/permitted-contents"), None);

    let skipped = Url::from_file_path(root.join("src/legacy/skip.vue")).unwrap();
    let (skipped_config, _) = state.linter_settings_for_uri(&skipped).unwrap();
    assert_eq!(skipped_config.rules.get("vue/permitted-contents"), None);

    let ignored = Url::from_file_path(root.join("src/generated/table.vue")).unwrap();
    assert!(state.linter_settings_for_uri(&ignored).is_none());
}

fn folder_with_config(parent: &std::path::Path, name: &str, config: &str) -> std::path::PathBuf {
    let dir = parent.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("vize.config.json"), config).unwrap();
    dir
}

#[test]
fn documents_resolve_their_own_folder_config_regardless_of_order() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let parent =
        std::env::temp_dir().join(cstr!("vize-folder-configs-{}-{nonce}", std::process::id()));
    let strict = parent.join("strict-root");
    std::fs::create_dir_all(&strict).unwrap();
    let relaxed = folder_with_config(
        &parent,
        "relaxed-root",
        r#"{ "linter": { "rules": { "vue/require-v-for-key": "off" } } }"#,
    );

    for roots in [
        vec![strict.clone(), relaxed.clone()],
        vec![relaxed.clone(), strict.clone()],
    ] {
        let state = ServerState::new();
        state.set_workspace_folders(roots);

        let strict_uri = Url::from_file_path(strict.join("List.vue")).unwrap();
        let (strict_config, _) = state.linter_settings_for_uri(&strict_uri).unwrap();
        assert_eq!(strict_config.rules.get("vue/require-v-for-key"), None);

        let relaxed_uri = Url::from_file_path(relaxed.join("List.vue")).unwrap();
        let (relaxed_config, _) = state.linter_settings_for_uri(&relaxed_uri).unwrap();
        assert_eq!(
            relaxed_config.rules.get("vue/require-v-for-key"),
            Some(&LintRuleSeverity::Off),
        );
    }

    let _ = std::fs::remove_dir_all(parent);
}

#[test]
fn removed_folders_drop_their_context_and_outside_documents_use_globals() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let parent =
        std::env::temp_dir().join(cstr!("vize-folder-removal-{}-{nonce}", std::process::id()));
    let relaxed = folder_with_config(
        &parent,
        "relaxed-root",
        r#"{ "linter": { "rules": { "vue/require-v-for-key": "off" } } }"#,
    );

    let state = ServerState::new();
    state.set_workspace_folders(vec![relaxed.clone()]);
    let uri = Url::from_file_path(relaxed.join("List.vue")).unwrap();
    let (config, _) = state.linter_settings_for_uri(&uri).unwrap();
    assert_eq!(
        config.rules.get("vue/require-v-for-key"),
        Some(&LintRuleSeverity::Off),
    );

    state.update_workspace_folders(Vec::new(), std::slice::from_ref(&relaxed));
    let (config, _) = state.linter_settings_for_uri(&uri).unwrap();
    assert_eq!(config.rules.get("vue/require-v-for-key"), None);

    let _ = std::fs::remove_dir_all(parent);
}

#[test]
fn workspace_folder_changes_select_only_open_documents_under_changed_roots() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let parent = std::env::temp_dir().join(cstr!(
        "vize-folder-revalidation-{}-{nonce}",
        std::process::id()
    ));
    let added_root = parent.join("added");
    let untouched_root = parent.join("untouched");
    std::fs::create_dir_all(&added_root).unwrap();
    std::fs::create_dir_all(&untouched_root).unwrap();

    let affected = Url::from_file_path(added_root.join("Affected.vue")).unwrap();
    let untouched = Url::from_file_path(untouched_root.join("Untouched.vue")).unwrap();
    let state = ServerState::new();
    state
        .documents
        .open(affected.clone(), "<template />".into(), 1, "vue".into());
    state
        .documents
        .open(untouched, "<template />".into(), 1, "vue".into());

    let selected = state.apply_workspace_folders_change(&WorkspaceFoldersChangeEvent {
        added: vec![WorkspaceFolder {
            uri: Url::from_file_path(&added_root).unwrap(),
            name: "added".into(),
        }],
        removed: Vec::new(),
    });

    assert_eq!(selected, vec![affected]);
    let _ = std::fs::remove_dir_all(parent);
}
