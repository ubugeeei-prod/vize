use std::fs;
use std::sync::Arc;

use tower_lsp::lsp_types::{Range, Url, WorkspaceEdit};
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use super::RenameService;
use crate::ide::IdeContext;
use crate::server::ServerState;

#[test]
fn canonical_event_rename_covers_static_declaration_variants_without_name_sweeps() {
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

        let variants = [
            (
                "RecordChild",
                r#"<script setup lang="ts">
defineEmits<{ saveItem: [id: string] }>();
</script>
"#,
                "<RecordChild @save-item",
            ),
            (
                "CallChild",
                r#"<script setup lang="ts">
defineEmits<{ (event: 'saveItem', id: string): void }>();
</script>
"#,
                "<CallChild @save-item",
            ),
            (
                "ArrayChild",
                r#"<script setup lang="ts">
defineEmits(['saveItem']);
</script>
"#,
                "<ArrayChild @save-item",
            ),
            (
                "ObjectChild",
                r#"<script setup lang="ts">
defineEmits({ saveItem: (id: string) => id.length > 0 });
</script>
"#,
                "<ObjectChild @save-item",
            ),
            (
                "GenericChild",
                r#"<script setup lang="ts" generic="T extends string = string">
defineProps<{ value: T }>();
defineEmits<{ saveItem: [id: T] }>();
</script>
"#,
                "<GenericChild value=\"chosen\" @save-item",
            ),
        ];
        let parent_source = r#"<script setup lang="ts">
import RecordChild from "./RecordChild.vue";
import CallChild from "./CallChild.vue";
import ArrayChild from "./ArrayChild.vue";
import ObjectChild from "./ObjectChild.vue";
import GenericChild from "./GenericChild.vue";
const handleSave = (id: string) => id;
</script>
<template>💥
  <RecordChild @save-item="handleSave" />
  <CallChild @save-item="handleSave" />
  <ArrayChild @save-item="handleSave" />
  <ObjectChild @save-item="handleSave" />
  <GenericChild value="chosen" @save-item="handleSave" />
</template>
"#;
        let parent_path = src.join("Parent.vue");
        fs::write(&parent_path, parent_source).expect("parent");
        let parent_uri = Url::from_file_path(&parent_path).expect("parent uri");
        let state = ServerState::new();
        state.set_workspace_root(project_root.clone());
        state.documents.open(
            parent_uri.clone(),
            parent_source.to_string(),
            1,
            "vue".to_string(),
        );
        state.update_virtual_docs(&parent_uri, parent_source);

        let mut children = Vec::new();
        for (name, source, parent_marker) in variants {
            let path = src.join(name).with_extension("vue");
            fs::write(&path, source).expect("child");
            let uri = Url::from_file_path(&path).expect("child uri");
            state
                .documents
                .open(uri.clone(), source.to_string(), 1, "vue".to_string());
            state.update_virtual_docs(&uri, source);
            children.push((uri, source, parent_marker));
        }

        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(tsgo_path),
            working_dir: Some(project_root),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.expect("tsgo session");

        for (child_uri, child_source, parent_marker) in &children {
            let child_offset = child_source.find("saveItem").expect("child event") + 2;
            let child_ctx = IdeContext::new(&state, child_uri, child_offset).expect("child ctx");
            let from_child = RenameService::rename_with_corsa(
                &child_ctx,
                "next-event",
                Some(Arc::clone(&bridge)),
            )
            .await
            .expect("rename from child declaration");
            assert_variant_edit(
                &from_child,
                (&parent_uri, parent_source, parent_marker),
                (child_uri, child_source),
            );

            let parent_start = parent_source.find(parent_marker).expect("parent event")
                + parent_marker.find("save-item").expect("event in marker");
            let parent_ctx =
                IdeContext::new(&state, &parent_uri, parent_start + 2).expect("parent ctx");
            let from_parent = RenameService::rename_with_corsa(
                &parent_ctx,
                "next-event",
                Some(Arc::clone(&bridge)),
            )
            .await
            .expect("rename from parent usage");
            assert_variant_edit(
                &from_parent,
                (&parent_uri, parent_source, parent_marker),
                (child_uri, child_source),
            );
        }
        bridge.shutdown().await.expect("shutdown");
    });
}

fn assert_variant_edit(edit: &WorkspaceEdit, parent: (&Url, &str, &str), child: (&Url, &str)) {
    let changes = edit.changes.as_ref().expect("plain workspace changes");
    assert_eq!(changes.len(), 2, "only the linked component: {changes:#?}");
    let parent_edits = changes.get(parent.0).expect("parent edit");
    assert_eq!(parent_edits.len(), 1, "no same-name sweep: {changes:#?}");
    let expected_start = parent.1.find(parent.2).expect("parent marker")
        + parent.2.find("save-item").expect("event in marker");
    assert_eq!(
        parent_edits[0].range,
        offset_range(parent.1, expected_start, "save-item".len())
    );
    assert_eq!(parent_edits[0].new_text, "next-event");

    let child_edits = changes.get(child.0).expect("child edit");
    assert!(child_edits.iter().any(|edit| {
        authored_text(child.1, edit.range) == "saveItem" && edit.new_text == "nextEvent"
    }));
    assert!(changes.keys().all(|uri| !uri.path().ends_with(".vue.ts")));
}

fn offset_range(source: &str, start: usize, len: usize) -> Range {
    let (start_line, start_character) = crate::ide::offset_to_position(source, start);
    let (end_line, end_character) = crate::ide::offset_to_position(source, start + len);
    Range::new(
        tower_lsp::lsp_types::Position::new(start_line, start_character),
        tower_lsp::lsp_types::Position::new(end_line, end_character),
    )
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
