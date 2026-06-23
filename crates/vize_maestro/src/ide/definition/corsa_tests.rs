use std::fs;

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Url};

use super::DefinitionService;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn definition_with_corsa_resolves_component_prop_attribute_to_child_prop() {
    crate::runtime::block_on(async {
        let Some(corsa_path) = resolve_tsgo_binary() else {
            return;
        };
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(
            dir.path().join("tsconfig.json"),
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
        .unwrap();
        fs::write(
            src.join("vue.d.ts"),
            r#"declare module "vue" {
  export type DefineComponent<P = any, _B = any, _D = any> = { new(): { $props: P } };
  export interface Ref<T = unknown> { value: T }
  export interface ShallowRef<T = unknown> extends Ref<T> {}
}
"#,
        )
        .unwrap();

        let child_source = r#"<script setup lang="ts">
defineProps<{
  message: string
}>()
</script>

<template><span /></template>
"#;
        let child_path = src.join("Child.vue");
        fs::write(&child_path, child_source).unwrap();

        let parent_source = r#"<script setup lang="ts">
import Child from './Child.vue'
const msg = 'hello'
</script>

<template>
  <Child :message="msg" />
</template>
"#;
        let parent_path = src.join("Parent.vue");
        fs::write(&parent_path, parent_source).unwrap();

        let uri = Url::from_file_path(&parent_path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(dir.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), parent_source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, parent_source);

        let bridge = std::sync::Arc::new(vize_canon::CorsaBridge::with_config(
            vize_canon::CorsaBridgeConfig {
                corsa_path: Some(corsa_path),
                working_dir: Some(dir.path().to_path_buf()),
                timeout_ms: 30_000,
                ..Default::default()
            },
        ));
        bridge.spawn().await.unwrap();

        let tag_offset = parent_source.find("<Child").unwrap() + 1;
        let tag_ctx = IdeContext::new(&state, &uri, tag_offset).unwrap();
        let tag_response = DefinitionService::definition_with_corsa(&tag_ctx, Some(bridge.clone()))
            .await
            .unwrap();
        let tag_location = scalar_location(tag_response);
        assert_eq!(
            tag_location
                .uri
                .to_file_path()
                .unwrap()
                .canonicalize()
                .unwrap(),
            child_path.canonicalize().unwrap()
        );

        let offset = parent_source.find(":message").unwrap() + 1;
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let response = DefinitionService::definition_with_corsa(&ctx, Some(bridge.clone()))
            .await
            .unwrap();
        let _ = bridge.shutdown().await;

        let location = scalar_location(response);
        let expected_offset = child_source.find("message: string").unwrap();
        let (line, character) = crate::ide::offset_to_position(child_source, expected_offset);

        assert_eq!(
            location.uri.to_file_path().unwrap().canonicalize().unwrap(),
            child_path.canonicalize().unwrap()
        );
        assert_eq!(location.range.start.line, line);
        assert_eq!(location.range.start.character, character);
    });
}

fn scalar_location(response: GotoDefinitionResponse) -> Location {
    match response {
        GotoDefinitionResponse::Scalar(location) => location,
        GotoDefinitionResponse::Array(mut locations) => {
            assert_eq!(locations.len(), 1);
            locations.remove(0)
        }
        GotoDefinitionResponse::Link(_) => panic!("expected location result"),
    }
}

fn resolve_tsgo_binary() -> Option<std::path::PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)?;
    for candidate in [
        workspace_root.parent()?.join("corsa-bind/.cache/tsgo"),
        workspace_root
            .parent()?
            .join("corsa-bind/ref/corsa-upstream/.cache/tsgo"),
        workspace_root.join("node_modules/.bin/tsgo"),
    ] {
        if candidate.exists() {
            return Some(candidate);
        }
    }

    vize_carton::corsa_resolver::discover_corsa_in_ancestors(workspace_root)
}
