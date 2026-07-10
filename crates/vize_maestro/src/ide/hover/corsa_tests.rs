use std::fs;

use tower_lsp::lsp_types::{Hover, HoverContents, MarkedString, Url};

use super::HoverService;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn hover_with_corsa_resolves_component_prop_attribute() {
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
        fs::write(
            src.join("Child.vue"),
            r#"<script setup lang="ts">
defineProps<{
  message: string
}>()
</script>
<template><span /></template>
"#,
        )
        .unwrap();

        let source = r#"<script setup lang="ts">
import Child from './Child.vue'
const msg = 'hello'
</script>

<template>
  <Child :message="msg" />
</template>
"#;
        let source_path = src.join("Parent.vue");
        fs::write(&source_path, source).unwrap();

        let uri = Url::from_file_path(&source_path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(dir.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);

        let bridge = std::sync::Arc::new(vize_canon::CorsaBridge::with_config(
            vize_canon::CorsaBridgeConfig {
                corsa_path: Some(corsa_path),
                working_dir: Some(dir.path().to_path_buf()),
                timeout_ms: 30_000,
                ..Default::default()
            },
        ));
        bridge.spawn().await.unwrap();

        let offset = source.find(":message").unwrap() + 1;
        let ctx = IdeContext::new(&state, &uri, offset).unwrap();
        let hover = HoverService::hover_with_corsa(&ctx, Some(bridge.clone()))
            .await
            .unwrap();
        let _ = bridge.shutdown().await;

        let value = hover_markdown(hover);
        assert!(value.contains("message"), "{value}");
        assert!(value.contains("string"), "{value}");
    });
}

fn hover_markdown(hover: Hover) -> String {
    match hover.contents {
        HoverContents::Markup(content) => content.value,
        HoverContents::Scalar(value) => marked_string_value(value),
        HoverContents::Array(parts) => parts
            .into_iter()
            .map(marked_string_value)
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

fn marked_string_value(value: MarkedString) -> String {
    match value {
        MarkedString::String(value) => value,
        MarkedString::LanguageString(value) => value.value,
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
