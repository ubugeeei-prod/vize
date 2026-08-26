use std::{fs, sync::Arc};

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Url};
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use super::TypeDefinitionService;
use crate::{ide::IdeContext, server::ServerState};

#[test]
fn canonical_type_definition_maps_template_binding_to_authored_interface() {
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

        let source = r#"<script setup lang="ts">
interface Product {
  name: string
}
const product = {} as Product
</script>
<template>{{ product.name }}</template>
"#;
        let path = src.join("Card.vue");
        fs::write(&path, source).expect("source");
        let uri = Url::from_file_path(&path).expect("source uri");
        let state = ServerState::new();
        state.set_workspace_root(project.path().to_path_buf());
        state
            .documents
            .open(uri.clone(), source.to_string(), 1, "vue".to_string());
        state.update_virtual_docs(&uri, source);
        let offset = source
            .rfind("product.name")
            .expect("template product binding")
            + "pro".len();
        let ctx = IdeContext::new(&state, &uri, offset).expect("context");
        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(tsgo_path),
            working_dir: Some(project.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.expect("tsgo session");

        let response =
            TypeDefinitionService::type_definition_with_corsa(&ctx, Some(bridge.clone()))
                .await
                .expect("type definition");
        bridge.shutdown().await.expect("shutdown");

        let location = scalar_location(response);
        assert_eq!(location.uri, uri);
        assert_eq!(authored_text(source, &location), "Product");
        assert!(
            !location.uri.path().ends_with(".vue.ts"),
            "generated URI leaked: {location:#?}",
        );
    });
}

fn scalar_location(response: GotoDefinitionResponse) -> Location {
    match response {
        GotoDefinitionResponse::Scalar(location) => location,
        GotoDefinitionResponse::Array(mut locations) => {
            assert_eq!(locations.len(), 1, "{locations:#?}");
            locations.remove(0)
        }
        GotoDefinitionResponse::Link(_) => panic!("expected location result"),
    }
}

fn authored_text<'a>(source: &'a str, location: &Location) -> &'a str {
    let start = crate::ide::position_to_offset(
        source,
        location.range.start.line,
        location.range.start.character,
    )
    .expect("source start");
    let end = crate::ide::position_to_offset(
        source,
        location.range.end.line,
        location.range.end.character,
    )
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
