use std::{fs, sync::Arc};

use tower_lsp::lsp_types::{PrepareRenameResponse, Range, TextEdit, Url};
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use super::RenameService;
use crate::{ide::IdeContext, server::ServerState};

mod component_props;
mod package_routes;
#[test]
fn canonical_rename_edits_authored_cross_vue_files() {
    crate::runtime::block_on(async {
        let Some(tsgo_path) = resolve_tsgo_binary() else {
            return;
        };
        let project = tempfile::TempDir::new().expect("temp project");
        let project_root = project
            .path()
            .canonicalize()
            .expect("canonical project root");
        let src = project_root.join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(
            project_root.join("tsconfig.json"),
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
        state.set_workspace_root(project_root.clone());
        for (uri, source) in [(&parent_uri, parent_source), (&child_uri, child_source)] {
            state
                .documents
                .open(uri.clone(), source.to_string(), 1, "vue".to_string());
            state.update_virtual_docs(uri, source);
        }
        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(tsgo_path),
            working_dir: Some(project_root),
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

#[test]
fn canonical_component_event_rename_preserves_camel_and_kebab_sites() {
    crate::runtime::block_on(async {
        let Some(tsgo_path) = resolve_tsgo_binary() else {
            return;
        };
        let project = tempfile::TempDir::new().expect("temp project");
        let project_root = project
            .path()
            .canonicalize()
            .expect("canonical project root");
        let src = project_root.join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(
            project_root.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
        )
        .expect("tsconfig");

        let child_source = r#"<script setup lang="ts">
defineEmits<{ saveItem: [id: string] }>();
</script>
"#;
        let parent_source = r#"<script setup lang="ts">
import Child from "./Child.vue";
const handleSave = (id: string) => id;
</script>
<template>💥 <Child @save-item="handleSave" /></template>
"#;
        let child_path = src.join("Child.vue");
        let parent_path = src.join("Parent.vue");
        fs::write(&child_path, child_source).expect("child");
        fs::write(&parent_path, parent_source).expect("parent");
        let child_uri = Url::from_file_path(&child_path).expect("child uri");
        let parent_uri = Url::from_file_path(&parent_path).expect("parent uri");
        let state = ServerState::new();
        state.set_workspace_root(project_root.clone());
        for (uri, source) in [(&child_uri, child_source), (&parent_uri, parent_source)] {
            state
                .documents
                .open(uri.clone(), source.to_string(), 1, "vue".to_string());
            state.update_virtual_docs(uri, source);
        }
        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(tsgo_path),
            working_dir: Some(project_root),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.expect("tsgo session");

        let event_start = parent_source.find("save-item").expect("parent event");
        for relative in 0.."save-item".len() {
            let cursor_ctx =
                IdeContext::new(&state, &parent_uri, event_start + relative).expect("cursor ctx");
            let prepared =
                RenameService::prepare_rename_with_corsa(&cursor_ctx, Some(Arc::clone(&bridge)))
                    .await
                    .expect("prepare event rename at every cursor");
            assert_eq!(
                authored_text(parent_source, prepare_range(prepared)),
                "save-item"
            );
        }
        let parent_offset = event_start + 2;
        let parent_ctx = IdeContext::new(&state, &parent_uri, parent_offset).expect("parent ctx");
        let prepared =
            RenameService::prepare_rename_with_corsa(&parent_ctx, Some(Arc::clone(&bridge)))
                .await
                .expect("prepare event rename");
        assert_eq!(
            authored_text(parent_source, prepare_range(prepared)),
            "save-item"
        );
        let from_parent =
            RenameService::rename_with_corsa(&parent_ctx, "nextItem", Some(Arc::clone(&bridge)))
                .await
                .expect("rename from parent");
        assert_component_event_edits(
            &from_parent,
            (&parent_uri, parent_source, "save-item", "next-item"),
            (&child_uri, child_source, "saveItem", "nextItem"),
        );
        let kebab_from_parent =
            RenameService::rename_with_corsa(&parent_ctx, "next-event", Some(Arc::clone(&bridge)))
                .await
                .expect("kebab rename from parent");
        assert_component_event_edits(
            &kebab_from_parent,
            (&parent_uri, parent_source, "save-item", "next-event"),
            (&child_uri, child_source, "saveItem", "nextEvent"),
        );

        let child_offset = child_source.find("saveItem").expect("child event") + 2;
        let child_ctx = IdeContext::new(&state, &child_uri, child_offset).expect("child ctx");
        let from_child =
            RenameService::rename_with_corsa(&child_ctx, "anotherEvent", Some(Arc::clone(&bridge)))
                .await
                .expect("rename from child");
        assert_component_event_edits(
            &from_child,
            (&parent_uri, parent_source, "save-item", "another-event"),
            (&child_uri, child_source, "saveItem", "anotherEvent"),
        );
        let kebab_from_child =
            RenameService::rename_with_corsa(&child_ctx, "final-event", Some(Arc::clone(&bridge)))
                .await
                .expect("kebab rename from child");
        bridge.shutdown().await.expect("shutdown");
        assert_component_event_edits(
            &kebab_from_child,
            (&parent_uri, parent_source, "save-item", "final-event"),
            (&child_uri, child_source, "saveItem", "finalEvent"),
        );
    });
}

fn assert_component_event_edits(
    edit: &tower_lsp::lsp_types::WorkspaceEdit,
    parent: (&Url, &str, &str, &str),
    child: (&Url, &str, &str, &str),
) {
    let changes = edit.changes.as_ref().expect("plain workspace changes");
    for (uri, source, old_name, new_name) in [parent, child] {
        let edits = changes.get(uri).expect("component event edits");
        assert!(
            edits.iter().any(|edit| {
                authored_text(source, edit.range) == old_name && edit.new_text == new_name
            }),
            "missing {old_name} -> {new_name}: {changes:#?}"
        );
    }
    assert!(changes.keys().all(|uri| !uri.path().ends_with(".vue.ts")));
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

pub(super) fn resolve_tsgo_binary() -> Option<std::path::PathBuf> {
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
