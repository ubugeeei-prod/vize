use std::fs;
use std::sync::Arc;

use tower_lsp::lsp_types::{Location, Url};
use vize_canon::{CorsaBridge, CorsaBridgeConfig};

use super::super::ReferencesService;
use crate::ide::IdeContext;
use crate::server::ServerState;

#[path = "corsa_tests/component_props.rs"]
mod component_props;
#[path = "corsa_tests/package_routes.rs"]
mod package_routes;
#[path = "corsa_tests/project_surface.rs"]
mod project_surface;

#[test]
fn canonical_references_cross_vue_files_and_honor_include_declaration() {
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
        fs::write(
            src.join("vue.d.ts"),
            r#"declare module "vue" {
  export type DefineComponent<P = any, _B = any, _D = any> = { new(): { $props: P } };
  export function defineComponent<T>(component: T): T;
}
"#,
        )
        .expect("vue types");

        let child_source = r#"<script lang="ts">
export const shared = 1
export const isolated = 2
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
        state.documents.open(
            parent_uri.clone(),
            parent_source.to_string(),
            1,
            "vue".to_string(),
        );
        state.update_virtual_docs(&parent_uri, parent_source);
        state.documents.open(
            child_uri.clone(),
            child_source.to_string(),
            1,
            "vue".to_string(),
        );
        state.update_virtual_docs(&child_uri, child_source);
        assert_eq!(state.open_importers(&child_uri), vec![parent_uri.clone()]);
        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(tsgo_path),
            working_dir: Some(project.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.expect("tsgo session");

        let query_offset = child_source.find("shared =").expect("export declaration") + 1;
        let ctx = IdeContext::new(&state, &child_uri, query_offset).expect("context");
        let without_declaration =
            ReferencesService::references_with_corsa(&ctx, false, Some(Arc::clone(&bridge)))
                .await
                .unwrap_or_default();
        let with_declaration =
            ReferencesService::references_with_corsa(&ctx, true, Some(Arc::clone(&bridge)))
                .await
                .expect("references with declaration");
        let isolated_offset = child_source
            .find("isolated =")
            .expect("isolated declaration")
            + 1;
        let isolated_ctx =
            IdeContext::new(&state, &child_uri, isolated_offset).expect("isolated context");
        let isolated_without_declaration = ReferencesService::references_with_corsa(
            &isolated_ctx,
            false,
            Some(Arc::clone(&bridge)),
        )
        .await
        .expect("an empty canonical result is authoritative");
        let template_offset = parent_source.find("{{ shared }}").expect("template use") + 3;
        let template_ctx =
            IdeContext::new(&state, &parent_uri, template_offset).expect("template context");
        let from_template = ReferencesService::references_with_corsa(
            &template_ctx,
            false,
            Some(Arc::clone(&bridge)),
        )
        .await
        .expect("template references");
        bridge.shutdown().await.expect("shutdown");

        assert!(
            with_declaration.len() > without_declaration.len(),
            "includeDeclaration must add the exported declaration: {with_declaration:#?}",
        );
        assert!(
            !without_declaration
                .iter()
                .any(|location| location.uri == child_uri),
            "the child contains only the excluded declaration: {without_declaration:#?}",
        );
        assert!(
            without_declaration
                .iter()
                .any(|location| location.uri == parent_uri),
            "references must use the unsaved child overlay across an encoded importer URI: {without_declaration:#?}",
        );
        let parent_without_declaration = without_declaration
            .iter()
            .filter(|location| location.uri == parent_uri)
            .collect::<Vec<_>>();
        assert_eq!(
            parent_without_declaration.len(),
            3,
            "the script, template, and style uses must survive while the import declaration stays excluded: {without_declaration:#?}",
        );
        for location in parent_without_declaration {
            assert_eq!(authored_text(parent_source, location), "shared");
        }
        let child_declaration = with_declaration
            .iter()
            .find(|location| location.uri == child_uri)
            .unwrap_or_else(|| panic!("authored child declaration: {with_declaration:#?}"));
        assert_eq!(authored_text(child_source, child_declaration), "shared");
        assert!(
            with_declaration
                .iter()
                .any(|location| location.uri == parent_uri),
            "references must cross into the open importer: {with_declaration:#?}",
        );
        assert_eq!(
            with_declaration
                .iter()
                .filter(|location| location.uri == parent_uri)
                .count(),
            4,
            "including declarations must retain the import plus script, template, and style uses: {with_declaration:#?}",
        );
        assert!(
            with_declaration
                .iter()
                .all(|location| !location.uri.path().ends_with(".vue.ts")),
            "canonical mirror URIs must never leak: {with_declaration:#?}",
        );
        assert!(
            isolated_without_declaration.is_empty(),
            "an isolated declaration has no references when declarations are excluded: {isolated_without_declaration:#?}",
        );
        assert_eq!(
            from_template.len(),
            3,
            "a template-local query must return only its script, template, and style uses: {from_template:#?}",
        );
        assert!(
            from_template
                .iter()
                .all(|location| location.uri == parent_uri
                    && authored_text(parent_source, location) == "shared"),
            "the linked query must not jump to the exported symbol or a shadowed spelling: {from_template:#?}",
        );
    });
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
    vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            project_root: Some(workspace_root),
            ..Default::default()
        },
    )
    .ok()
}
