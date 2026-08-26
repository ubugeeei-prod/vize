use std::fs;
use std::sync::Arc;

use tower_lsp::lsp_types::{PrepareRenameResponse, Range, Url, WorkspaceEdit};
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use super::RenameService;
use crate::ide::IdeContext;
use crate::server::ServerState;

#[test]
fn canonical_model_event_rename_preserves_update_prefix_and_authored_casing() {
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
export interface Props { value?: string }
defineProps<Props>();
defineModel<string>('title', { required: true });
</script>
"#;
        let parent_source = r#"<script setup lang="ts">
import Child from "./Child.vue";
import Other from "./Other.vue";
const modelValue = "";
const otherValue = "";
const handleUpdate = (value: string) => value;
</script>
<template>
  💥 <Child v-model:title="modelValue" />
  <Child @update:title="handleUpdate" />
  <Other v-model:title="otherValue" />
</template>
"#;
        let child_path = src.join("Child.vue");
        let other_path = src.join("Other.vue");
        let parent_path = src.join("Parent.vue");
        fs::write(&child_path, child_source).expect("child");
        fs::write(&other_path, child_source).expect("other");
        fs::write(&parent_path, parent_source).expect("parent");
        let child_uri = Url::from_file_path(&child_path).expect("child uri");
        let other_uri = Url::from_file_path(&other_path).expect("other uri");
        let parent_uri = Url::from_file_path(&parent_path).expect("parent uri");
        let state = ServerState::new();
        state.set_workspace_root(project_root.clone());
        for (uri, source) in [
            (&child_uri, child_source),
            (&other_uri, child_source),
            (&parent_uri, parent_source),
        ] {
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

        let parent_start = parent_source.find("update:title").expect("parent event");
        for cursor in parent_start..parent_start + "update:title".len() {
            let ctx = IdeContext::new(&state, &parent_uri, cursor).expect("parent prepare ctx");
            let prepared =
                RenameService::prepare_rename_with_corsa(&ctx, Some(Arc::clone(&bridge)))
                    .await
                    .expect("prepare model rename");
            assert_eq!(
                authored_text(parent_source, prepare_range(prepared)),
                "update:title"
            );
        }
        let parent_offset = parent_start + 8;
        let parent_ctx = IdeContext::new(&state, &parent_uri, parent_offset).expect("parent ctx");
        let from_parent = RenameService::rename_with_corsa(
            &parent_ctx,
            "update:headline",
            Some(Arc::clone(&bridge)),
        )
        .await
        .expect("rename model from parent");
        assert_model_edits(
            &from_parent,
            (
                &parent_uri,
                parent_source,
                "update:title",
                "update:headline",
            ),
            (&child_uri, child_source, "title", "headline"),
        );
        assert_authored_edit(
            &from_parent,
            &parent_uri,
            parent_source,
            "title",
            "headline",
        );
        assert_other_model_untouched(&from_parent, &parent_uri, &other_uri, parent_source);

        let child_start = child_source.find("'title'").expect("child model") + 1;
        for cursor in child_start..child_start + "title".len() {
            let ctx = IdeContext::new(&state, &child_uri, cursor).expect("child prepare ctx");
            let prepared =
                RenameService::prepare_rename_with_corsa(&ctx, Some(Arc::clone(&bridge)))
                    .await
                    .expect("prepare model declaration rename");
            assert_eq!(
                authored_text(child_source, prepare_range(prepared)),
                "title"
            );
        }
        let child_offset = child_start + 2;
        let child_ctx = IdeContext::new(&state, &child_uri, child_offset).expect("child ctx");
        let from_child = RenameService::rename_with_corsa(
            &child_ctx,
            "headline-value",
            Some(Arc::clone(&bridge)),
        )
        .await
        .expect("rename model from child");
        assert_model_edits(
            &from_child,
            (
                &parent_uri,
                parent_source,
                "update:title",
                "update:headline-value",
            ),
            (&child_uri, child_source, "title", "headlineValue"),
        );
        assert_authored_edit(
            &from_child,
            &parent_uri,
            parent_source,
            "title",
            "headline-value",
        );
        assert_other_model_untouched(&from_child, &parent_uri, &other_uri, parent_source);

        let usage_start =
            parent_source.find("model:title").expect("v-model usage") + "model:".len();
        for cursor in usage_start..usage_start + "title".len() {
            let ctx = IdeContext::new(&state, &parent_uri, cursor).expect("usage prepare ctx");
            let prepared =
                RenameService::prepare_rename_with_corsa(&ctx, Some(Arc::clone(&bridge)))
                    .await
                    .expect("prepare v-model rename");
            assert_eq!(
                authored_text(parent_source, prepare_range(prepared)),
                "title"
            );
        }
        let usage_offset = usage_start + 1;
        let usage_ctx = IdeContext::new(&state, &parent_uri, usage_offset).expect("usage ctx");
        let from_usage =
            RenameService::rename_with_corsa(&usage_ctx, "final-model", Some(Arc::clone(&bridge)))
                .await
                .expect("rename model from v-model");
        bridge.shutdown().await.expect("shutdown");
        assert_model_edits(
            &from_usage,
            (
                &parent_uri,
                parent_source,
                "update:title",
                "update:final-model",
            ),
            (&child_uri, child_source, "title", "finalModel"),
        );
        assert_authored_edit(
            &from_usage,
            &parent_uri,
            parent_source,
            "title",
            "final-model",
        );
        assert_other_model_untouched(&from_usage, &parent_uri, &other_uri, parent_source);
    });
}

fn assert_other_model_untouched(
    edit: &WorkspaceEdit,
    parent_uri: &Url,
    other_uri: &Url,
    parent_source: &str,
) {
    let changes = edit.changes.as_ref().expect("plain workspace changes");
    assert!(!changes.contains_key(other_uri), "{changes:#?}");
    let start = parent_source.rfind("model:title").expect("other model") + "model:".len();
    let (line, character) = crate::ide::offset_to_position(parent_source, start);
    let edits = changes.get(parent_uri).expect("parent edits");
    assert!(
        edits
            .iter()
            .all(|edit| edit.range.start.line != line || edit.range.start.character != character),
        "unrelated model was renamed: {changes:#?}"
    );
}

fn assert_authored_edit(
    edit: &WorkspaceEdit,
    uri: &Url,
    source: &str,
    old_name: &str,
    new_name: &str,
) {
    let changes = edit.changes.as_ref().expect("plain workspace changes");
    let edits = changes.get(uri).expect("authored model edits");
    assert!(
        edits.iter().any(|edit| {
            authored_text(source, edit.range) == old_name && edit.new_text == new_name
        }),
        "missing {old_name} -> {new_name}: {changes:#?}"
    );
}

fn assert_model_edits(
    edit: &WorkspaceEdit,
    parent: (&Url, &str, &str, &str),
    child: (&Url, &str, &str, &str),
) {
    let changes = edit.changes.as_ref().expect("plain workspace changes");
    for (uri, source, old_name, new_name) in [parent, child] {
        let edits = changes.get(uri).expect("model edits");
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
        PrepareRenameResponse::DefaultBehavior { .. } => panic!("expected concrete range"),
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
    vize_s0::corsa_resolver::resolve_corsa_executable(
        vize_s0::corsa_resolver::CorsaResolveRequest {
            project_root: Some(workspace_root),
            ..Default::default()
        },
    )
    .ok()
}
