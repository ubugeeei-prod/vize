#![allow(clippy::disallowed_methods)]

use std::fs;

use tower_lsp::lsp_types::FileRename;

use super::manual::collect_import_rename_edits;
use super::manual_tests::{file_uri, normalize_edit, test_dir};
use crate::server::ServerState;

#[test]
fn project_source_alias_rename_edits_work_without_tsconfig_paths() {
    let dir = test_dir();
    let root = dir.path();
    let views = root.join("src/views");
    let components = root.join("src/components");
    fs::create_dir_all(&views).unwrap();
    fs::create_dir_all(&components).unwrap();
    fs::write(root.join("package.json"), r#"{"type":"module"}"#).unwrap();

    let importer = views.join("docs.vue");
    fs::write(
        &importer,
        "<script setup>\nimport HighlightMessage from '@/components/__VizeOracleHighlightMessage.vue'\n</script>\n",
    )
    .unwrap();
    let copied = components.join("__VizeOracleHighlightMessage.vue");
    let renamed = components.join("__VizeOracleRenamedHighlightMessage.vue");
    fs::write(&copied, "<template />\n").unwrap();

    let state = ServerState::new();
    state.set_workspace_root(root.to_path_buf());

    let edit = collect_import_rename_edits(
        &state,
        &[FileRename {
            old_uri: file_uri(&copied),
            new_uri: file_uri(&renamed),
        }],
        true,
    )
    .expect("rename edit for project source alias without tsconfig paths");
    let normalized = normalize_edit(root, &edit);

    assert_eq!(
        normalized
            .pointer("/src~1views~1docs.vue/0/newText")
            .and_then(serde_json::Value::as_str),
        Some("@/components/__VizeOracleRenamedHighlightMessage.vue")
    );
}
