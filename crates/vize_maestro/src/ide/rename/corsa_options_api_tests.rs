use std::{fs, sync::Arc};

use tower_lsp::lsp_types::{PrepareRenameResponse, Range, Url};
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use super::RenameService;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn canonical_options_api_template_rename_includes_authored_declaration() {
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

        let source = r#"<template>
  <div :style="sliderLeft + '; width: 50%;'"></div>
</template>
<script lang="ts">
export default {
  computed: {
    sliderLeft () {
      return 'left: 0%'
    }
  }
}
</script>
"#;
        let path = src.join("Tab.vue");
        fs::write(&path, source).expect("source");
        let uri = Url::from_file_path(&path).expect("uri");
        let state = ServerState::new();
        state.set_workspace_root(project_root.clone());
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);
        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(tsgo_path),
            working_dir: Some(project_root),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.expect("tsgo session");

        let query_offset = source.find("sliderLeft +").expect("template binding") + 1;
        let ctx = IdeContext::new(&state, &uri, query_offset).expect("context");
        let prepare = RenameService::prepare_rename_with_corsa(&ctx, Some(Arc::clone(&bridge)))
            .await
            .expect("prepare rename");
        assert_eq!(authored_text(source, prepare_range(prepare)), "sliderLeft");

        let edit =
            RenameService::rename_with_corsa(&ctx, "renamedSliderLeft", Some(Arc::clone(&bridge)))
                .await
                .expect("rename");
        bridge.shutdown().await.expect("shutdown");

        let changes = edit.changes.expect("plain workspace changes");
        assert_eq!(changes.keys().collect::<Vec<_>>(), [&uri]);
        let edits = changes.get(&uri).expect("SFC edits");
        assert_eq!(edits.len(), 2, "template and declaration edits: {edits:#?}");
        assert!(
            edits
                .iter()
                .all(|edit| edit.new_text == "renamedSliderLeft")
        );
        assert!(
            edits
                .iter()
                .any(|edit| authored_text(source, edit.range) == "sliderLeft"
                    && edit.range.start.line == 1),
            "missing template edit: {edits:#?}"
        );
        assert!(
            edits
                .iter()
                .any(|edit| authored_text(source, edit.range) == "sliderLeft"
                    && edit.range.start.line == 6),
            "missing Options API declaration edit: {edits:#?}"
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
