use std::{path::PathBuf, sync::Arc};

use tower_lsp::lsp_types::{CallHierarchyItem, Url};
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use super::CallHierarchyService;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn prepare_call_hierarchy_maps_script_setup_function_to_authored_source() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let project = tempfile::TempDir::new().expect("temp project");
        let src = project.path().join("src");
        std::fs::create_dir_all(&src).expect("src");
        std::fs::write(
            project.path().join("tsconfig.json"),
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

        let source = r#"<script setup lang="ts">
function run(value: string): string {
  return value
}
</script>
<template>
  <button @click="run('template')">{{ run('label') }}</button>
</template>
"#;
        let path = src.join("App.vue");
        std::fs::write(&path, source).expect("source");
        let uri = Url::from_file_path(&path).expect("source uri");
        let state = ServerState::new();
        state.set_workspace_root(project.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);
        let offset = source
            .find("run(value")
            .expect("script setup call hierarchy marker")
            + "r".len();
        let ctx = IdeContext::new(&state, &uri, offset).expect("context");
        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(corsa_path),
            working_dir: Some(project.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.expect("tsgo session");

        let items = CallHierarchyService::prepare_with_corsa(&ctx, Some(bridge.clone()))
            .await
            .expect("call hierarchy items");
        bridge.shutdown().await.expect("shutdown");

        let item = single_item(items);
        assert_eq!(item.uri, uri);
        assert_eq!(authored_text(source, item.selection_range), "run");
        assert!(
            !item.uri.path().ends_with(".vue.ts"),
            "generated URI leaked: {item:#?}",
        );
    });
}

#[test]
fn incoming_and_outgoing_call_hierarchy_maps_call_sites_to_authored_source() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let project = tempfile::TempDir::new().expect("temp project");
        let src = project.path().join("src");
        std::fs::create_dir_all(&src).expect("src");
        std::fs::write(
            project.path().join("tsconfig.json"),
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

        let source = r#"<script setup lang="ts">
function leaf(value: string): string {
  return value
}
function caller(): string {
  return leaf('setup')
}
</script>
<template>
  <button @click="caller()">{{ leaf('template') }}</button>
</template>
"#;
        let path = src.join("App.vue");
        std::fs::write(&path, source).expect("source");
        let uri = Url::from_file_path(&path).expect("source uri");
        let state = ServerState::new();
        state.set_workspace_root(project.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);
        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(corsa_path),
            working_dir: Some(project.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.expect("tsgo session");

        let leaf = prepare_item(source, &uri, &state, &bridge, "leaf(value").await;
        let caller = prepare_item(source, &uri, &state, &bridge, "caller():").await;
        let caller_ctx = context_at(source, &uri, &state, "caller():");
        let outgoing = CallHierarchyService::outgoing_calls_with_corsa(
            &caller_ctx,
            &caller,
            Some(bridge.clone()),
        )
        .await
        .expect("caller outgoing calls");
        assert!(
            outgoing.iter().any(|call| {
                call.to.name == "leaf"
                    && call
                        .from_ranges
                        .iter()
                        .any(|range| authored_text(source, *range) == "leaf")
            }),
            "caller should expose the authored leaf call site: {outgoing:#?}",
        );

        let leaf_ctx = context_at(source, &uri, &state, "leaf(value");
        let incoming =
            CallHierarchyService::incoming_calls_with_corsa(&leaf_ctx, &leaf, Some(bridge.clone()))
                .await
                .expect("leaf incoming calls");
        assert!(
            incoming.iter().any(|call| {
                call.from.name == "caller"
                    && call
                        .from_ranges
                        .iter()
                        .any(|range| authored_text(source, *range) == "leaf")
            }),
            "leaf should expose caller through authored ranges: {incoming:#?}",
        );

        bridge.shutdown().await.expect("shutdown");
    });
}

async fn prepare_item(
    source: &str,
    uri: &Url,
    state: &ServerState,
    bridge: &Arc<CorsaBridge>,
    marker: &str,
) -> CallHierarchyItem {
    let ctx = context_at(source, uri, state, marker);
    let items = CallHierarchyService::prepare_with_corsa(&ctx, Some(bridge.clone()))
        .await
        .expect("call hierarchy items");
    single_item(items)
}

fn context_at<'a>(
    source: &'a str,
    uri: &'a Url,
    state: &'a ServerState,
    marker: &str,
) -> IdeContext<'a> {
    let offset = source.find(marker).expect("marker") + 1;
    IdeContext::new(state, uri, offset).expect("context")
}

fn single_item(mut items: Vec<CallHierarchyItem>) -> CallHierarchyItem {
    assert_eq!(items.len(), 1, "{items:#?}");
    items.remove(0)
}

fn authored_text(source: &str, range: tower_lsp::lsp_types::Range) -> &str {
    let start = crate::ide::position_to_offset(source, range.start.line, range.start.character)
        .expect("source start");
    let end = crate::ide::position_to_offset(source, range.end.line, range.end.character)
        .expect("source end");
    &source[start..end]
}

fn resolve_tsgo_binary() -> Option<PathBuf> {
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
