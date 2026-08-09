use std::fs;
use std::sync::Arc;

use tower_lsp::lsp_types::{PrepareRenameResponse, Range, TextEdit, Url};
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use super::RenameService;
use crate::ide::IdeContext;
use crate::server::ServerState;

#[test]
fn canonical_rename_edits_authored_cross_vue_files() {
    crate::runtime::block_on(async {
        let Some(tsgo_path) = resolve_tsgo_binary() else {
            return;
        };
        let project = tempfile::TempDir::new().expect("temp project");
        let src = project.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(
            project.path().join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
        )
        .expect("tsconfig");

        let child_source = r#"<script lang="ts">
export const shared = 1
export default {}
</script>
<template><span /></template>
"#;
        let child_path = src.join("Child View.vue");
        fs::write(
            &child_path,
            r#"<script lang="ts">
export const diskOnly = 0
export default {}
</script>
"#,
        )
        .expect("stale child on disk");
        let parent_source = r#"<script setup lang="ts">
import { shared } from './Child View.vue'
const local = shared
function unrelated() { const shared = 0; return shared }
</script>
<template>💥 {{ shared }} {{ local }} <i v-for="shared in [1]">{{ shared }}</i></template>
<style>.root { color: v-bind(shared) }</style>
"#;
        let parent_path = src.join("Parent View.vue");
        fs::write(&parent_path, parent_source).expect("parent");

        let parent_uri = Url::from_file_path(&parent_path).expect("parent uri");
        let child_uri = Url::from_file_path(&child_path).expect("child uri");
        let state = ServerState::new();
        state.set_workspace_root(project.path().to_path_buf());
        for (uri, source) in [(&parent_uri, parent_source), (&child_uri, child_source)] {
            state
                .documents
                .open(uri.clone(), source.to_string(), 1, "vue".to_string());
            state.update_virtual_docs(uri, source);
        }
        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(tsgo_path),
            working_dir: Some(project.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.expect("tsgo session");

        let query_offset = child_source.find("shared =").expect("declaration") + 1;
        let ctx = IdeContext::new(&state, &child_uri, query_offset).expect("context");
        let prepare = RenameService::prepare_rename_with_corsa(&ctx, Some(Arc::clone(&bridge)))
            .await
            .expect("prepare rename");
        assert_eq!(
            authored_text(child_source, prepare_range(prepare)),
            "shared"
        );

        let edit = RenameService::rename_with_corsa(&ctx, "renamed", Some(Arc::clone(&bridge)))
            .await
            .expect("rename");
        let template_offset = parent_source.find("{{ shared }}").expect("template use") + 3;
        let template_ctx =
            IdeContext::new(&state, &parent_uri, template_offset).expect("template context");
        let template_prepare =
            RenameService::prepare_rename_with_corsa(&template_ctx, Some(Arc::clone(&bridge)))
                .await
                .expect("template prepare rename");
        assert_eq!(
            authored_text(parent_source, prepare_range(template_prepare)),
            "shared",
        );
        let local_edit = RenameService::rename_with_corsa(
            &template_ctx,
            "renamedLocal",
            Some(Arc::clone(&bridge)),
        )
        .await
        .expect("template rename");
        bridge.shutdown().await.expect("shutdown");

        let changes = edit.changes.expect("plain workspace changes");
        assert_eq!(changes.len(), 2, "only authored Vue files: {changes:#?}");
        assert_authored_edits(
            child_source,
            changes.get(&child_uri).expect("child edits"),
            1,
        );
        assert_authored_edits(
            parent_source,
            changes.get(&parent_uri).expect("parent edits"),
            4,
        );
        assert!(
            changes.keys().all(|uri| !uri.path().ends_with(".vue.ts")),
            "canonical mirror URIs must never leak: {changes:#?}",
        );

        let local_changes = local_edit.changes.expect("local workspace changes");
        assert_eq!(
            local_changes.keys().collect::<Vec<_>>(),
            [&parent_uri],
            "renaming the imported local from template must not rename the child export: {local_changes:#?}",
        );
        let local_edits = local_changes.get(&parent_uri).expect("parent local edits");
        assert_eq!(
            local_edits.len(),
            4,
            "import, script, template, and style edits",
        );
        assert!(
            local_edits
                .iter()
                .all(|edit| authored_text(parent_source, edit.range) == "shared"),
            "shadowed same-spelling bindings must remain untouched: {local_edits:#?}",
        );
        let mut replacements = local_edits
            .iter()
            .map(|edit| edit.new_text.as_str())
            .collect::<Vec<_>>();
        replacements.sort_unstable();
        assert_eq!(
            replacements,
            [
                "renamedLocal",
                "renamedLocal",
                "renamedLocal",
                "shared as renamedLocal",
            ],
            "the import edit must preserve the exported name while changing the local alias",
        );
    });
}

fn prepare_range(response: PrepareRenameResponse) -> Range {
    match response {
        PrepareRenameResponse::Range(range)
        | PrepareRenameResponse::RangeWithPlaceholder { range, .. } => range,
        PrepareRenameResponse::DefaultBehavior { .. } => panic!("expected an authored range"),
    }
}

fn assert_authored_edits(source: &str, edits: &[TextEdit], expected: usize) {
    assert_eq!(edits.len(), expected, "unexpected edits: {edits:#?}");
    for edit in edits {
        assert_eq!(edit.new_text, "renamed");
        assert_eq!(authored_text(source, edit.range), "shared");
    }
}

fn authored_text(source: &str, range: Range) -> &str {
    let start = crate::ide::position_to_offset(source, range.start.line, range.start.character)
        .expect("source start");
    let end = crate::ide::position_to_offset(source, range.end.line, range.end.character)
        .expect("source end");
    &source[start..end]
}

fn resolve_tsgo_binary() -> Option<std::path::PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }
    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)?;
    vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            project_root: Some(workspace_root),
            ..Default::default()
        },
    )
    .ok()
}
