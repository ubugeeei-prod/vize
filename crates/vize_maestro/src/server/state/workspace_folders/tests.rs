use tower_lsp::lsp_types::{Url, WorkspaceFolder, WorkspaceFoldersChangeEvent};
use vize_l0::{config::LintRuleSeverity, cstr};

use crate::server::ServerState;

#[test]
fn scoped_rules_and_global_ignores_match_document_paths() {
    let parent = crate::test_support::tempdir().unwrap();
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
    let (legacy_config, _, _) = state.linter_settings_for_uri(&legacy).unwrap();
    assert_eq!(
        legacy_config.rules.get("vue/permitted-contents"),
        Some(&LintRuleSeverity::Off)
    );

    let other = Url::from_file_path(root.join("src/other.vue")).unwrap();
    let (other_config, _, _) = state.linter_settings_for_uri(&other).unwrap();
    assert_eq!(other_config.rules.get("vue/permitted-contents"), None);

    let skipped = Url::from_file_path(root.join("src/legacy/skip.vue")).unwrap();
    let (skipped_config, _, _) = state.linter_settings_for_uri(&skipped).unwrap();
    assert_eq!(skipped_config.rules.get("vue/permitted-contents"), None);

    let ignored = Url::from_file_path(root.join("src/generated/table.vue")).unwrap();
    assert!(state.linter_settings_for_uri(&ignored).is_none());
}

/// #9686: a workspace opened through a symlink still applies its scoped
/// rules and ignores; config bases are physical, document paths are not.
#[cfg(unix)]
#[test]
fn scoped_rules_and_ignores_apply_through_a_symlinked_workspace() {
    let parent = crate::test_support::tempdir().unwrap();
    let physical = folder_with_config(
        parent.path(),
        "project",
        r#"{
  "entries": [{"files":["src/legacy/**/*.vue"], "linter":{"rules":{"vue/permitted-contents":"off"}}}],
  "ignores": ["src/generated/**"]
}"#,
    );
    let root = parent.path().join("alias");
    std::os::unix::fs::symlink(&physical, &root).unwrap();
    let state = ServerState::new();
    state.apply_initialize_workspace_folders(None, Some(&root));

    let legacy = Url::from_file_path(root.join("src/legacy/table.vue")).unwrap();
    let (legacy_config, _, _) = state.linter_settings_for_uri(&legacy).unwrap();
    assert_eq!(
        legacy_config.rules.get("vue/permitted-contents"),
        Some(&LintRuleSeverity::Off)
    );
    let ignored = Url::from_file_path(root.join("src/generated/table.vue")).unwrap();
    assert!(state.linter_settings_for_uri(&ignored).is_none());
}

fn folder_with_config(parent: &std::path::Path, name: &str, config: &str) -> std::path::PathBuf {
    let dir = parent.join(name);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("vite.config.mjs"),
        vize_l0::cstr!("export default {{ vue: {} }};", config).as_bytes(),
    )
    .unwrap();
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
        let (strict_config, _, _) = state.linter_settings_for_uri(&strict_uri).unwrap();
        assert_eq!(strict_config.rules.get("vue/require-v-for-key"), None);

        let relaxed_uri = Url::from_file_path(relaxed.join("List.vue")).unwrap();
        let (relaxed_config, _, _) = state.linter_settings_for_uri(&relaxed_uri).unwrap();
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
    let (config, _, _) = state.linter_settings_for_uri(&uri).unwrap();
    assert_eq!(
        config.rules.get("vue/require-v-for-key"),
        Some(&LintRuleSeverity::Off),
    );

    state.update_workspace_folders(Vec::new(), std::slice::from_ref(&relaxed));
    let (config, _, _) = state.linter_settings_for_uri(&uri).unwrap();
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

#[test]
fn invalid_project_configuration_refuses_lint_without_borrowing_a_sibling_policy() {
    let parent = crate::test_support::tempdir().unwrap();
    let valid = folder_with_config(
        parent.path(),
        "valid",
        r#"{"linter":{"rules":{"a11y/alt-text":"off"}}}"#,
    );
    let invalid = parent.path().join("invalid");
    std::fs::create_dir_all(&invalid).unwrap();
    std::fs::write(
        invalid.join("vite.config.mjs"),
        "export default { vue: { formatter: { printWidth: 'invalid' } } };",
    )
    .unwrap();
    let state = ServerState::new();
    state.set_workspace_folders(vec![valid.clone(), invalid.clone()]);
    assert!(
        state
            .linter_settings_for_uri(&Url::from_file_path(invalid.join("App.vue")).unwrap())
            .is_none()
    );
    let (config, _, _) = state
        .linter_settings_for_uri(&Url::from_file_path(valid.join("App.vue")).unwrap())
        .unwrap();
    assert_eq!(
        config.rules.get("a11y/alt-text"),
        Some(&LintRuleSeverity::Off)
    );
}

#[test]
fn invalid_native_project_initialization_refuses_lint_until_valid_configuration_is_loaded() {
    let project = crate::test_support::tempdir().unwrap();
    let config = project.path().join("vite.config.mjs");
    std::fs::write(&config, "throw new Error('invalid configuration');").unwrap();
    let state = ServerState::new();
    let uri = Url::from_file_path(project.path().join("App.vue")).unwrap();
    state.load_workspace_config(project.path());
    assert!(state.linter_settings_for_uri(&uri).is_none());
    std::fs::write(
        &config,
        "export default { vue: { linter: { rules: { 'a11y/alt-text': 'off' } } } };",
    )
    .unwrap();
    state.load_workspace_config(project.path());
    let (loaded, _, _) = state.linter_settings_for_uri(&uri).unwrap();
    assert_eq!(
        loaded.rules.get("a11y/alt-text"),
        Some(&LintRuleSeverity::Off)
    );
    std::fs::write(&config, "throw new Error('invalid edit');").unwrap();
    state.load_workspace_config(project.path());
    let (retained, _, _) = state.linter_settings_for_uri(&uri).unwrap();
    assert_eq!(
        retained.rules.get("a11y/alt-text"),
        Some(&LintRuleSeverity::Off)
    );
}

#[test]
#[cfg(feature = "native")]
#[expect(
    clippy::disallowed_types,
    reason = "the asynchronous admission worker retains its shared editor owner"
)]
fn invalid_primary_initialization_loads_other_folders_on_the_same_worker() {
    let project = crate::test_support::tempdir().unwrap();
    let invalid = folder_with_config(project.path(), "invalid", "{formatter:{printWidth:'bad'}}");
    let valid = folder_with_config(
        project.path(),
        "valid",
        "{linter:{rules:{'a11y/alt-text':'off'}}}",
    );
    let state = std::sync::Arc::new(ServerState::new());
    state.set_workspace_root(invalid.clone());
    let folders = [invalid.clone(), valid.clone()].map(|root| WorkspaceFolder {
        uri: Url::from_file_path(&root).unwrap(),
        name: root.file_name().unwrap().to_string_lossy().into_owned(),
    });
    assert!(crate::runtime::block_on(
        state.load_workspace_config_async(&invalid, Some(&folders))
    ));
    let invalid_uri = Url::from_file_path(invalid.join("App.vue")).unwrap();
    assert!(state.linter_settings_for_uri(&invalid_uri).is_none());
    let valid_uri = Url::from_file_path(valid.join("App.vue")).unwrap();
    let (config, _, _) = state.linter_settings_for_uri(&valid_uri).unwrap();
    assert_eq!(
        config.rules.get("a11y/alt-text"),
        Some(&LintRuleSeverity::Off)
    );
}
