use std::fs;
use std::sync::Arc;

use tower_lsp::lsp_types::Url;
use vize_canon::{CorsaBridge, CorsaBridgeConfig};
use vize_s0::{String, cstr};

use super::{ReferencesService, authored_text, resolve_tsgo_binary};
use crate::ide::IdeContext;
use crate::server::ServerState;

// The bounded editor transport used to overflow while opening component 335.
// Keep this above that boundary so the regression test exercises chunked
// synchronization without paying the full 500-component benchmark cost in CI.
const COMPONENT_COUNT: usize = 336;

const PROBE_SOURCE: &str = r#"<script setup lang="ts">
import { ref } from 'vue'
import { sharedLabel } from './shared'
const probeLabel = ref(sharedLabel(0))
</script>
<template>{{ probeLabel.value }}</template>
"#;

fn component_source(index: usize) -> String {
    let mut source = cstr!(
        r#"<script setup lang="ts">
import {{ computed, ref }} from 'vue'
import {{ sharedLabel }} from './shared'
const caption = ref(sharedLabel({index}))
const doubled = computed(() => {index} * 2)
</script>
<template>{{{{ caption.value }}}} {{{{ doubled.value }}}}</template>
"#,
    );
    if index == 1 {
        source.push_str("<style scoped>.item { color: v-bind(sharedLabel) }</style>\n");
    }
    source
}

#[test]
fn canonical_references_search_closed_generated_component_surface() {
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
        let vue_package = project.path().join("node_modules/vue");
        fs::create_dir_all(&vue_package).expect("Vue package");
        fs::write(
            vue_package.join("package.json"),
            r#"{
  "name": "vue",
  "version": "3.5.0",
  "types": "./index.d.ts",
  "exports": { ".": { "types": "./index.d.ts" } }
}"#,
        )
        .expect("Vue manifest");
        fs::write(
            vue_package.join("index.d.ts"),
            r#"export declare function ref<T>(value: T): { value: T };
export declare function computed<T>(get: () => T): { value: T };
"#,
        )
        .expect("Vue declarations");
        let shared_source =
            "export function sharedLabel(index: number): string { return `item-${index}` }\n";
        let shared_path = src.join("shared.ts");
        fs::write(&shared_path, shared_source).expect("shared module");

        let probe_path = src.join("ScaleProbe.vue");
        fs::write(&probe_path, PROBE_SOURCE).expect("probe");
        let probe_uri = Url::from_file_path(&probe_path).expect("probe uri");
        let outside_path = project.path().join("ignored/Outside.vue");
        fs::create_dir_all(outside_path.parent().expect("outside parent")).expect("outside dir");
        fs::write(
            &outside_path,
            r#"<script setup lang="ts">
import { sharedLabel } from '../src/shared'
const outside = sharedLabel(999)
</script>
"#,
        )
        .expect("excluded component");
        let outside_uri = Url::from_file_path(outside_path).expect("outside uri");

        let components = (1..=COMPONENT_COUNT)
            .map(|index| {
                let name = cstr!("Comp{index:04}.vue");
                let path = src.join(name.as_str());
                let source = component_source(index);
                fs::write(&path, &source).expect("generated component");
                let uri = Url::from_file_path(path).expect("component uri");
                (uri, source)
            })
            .collect::<Vec<_>>();

        let state = ServerState::new();
        state.set_workspace_root(project.path().to_path_buf());
        state.documents.open(
            probe_uri.clone(),
            PROBE_SOURCE.to_string(),
            1,
            "vue".to_string(),
        );
        state.update_virtual_docs(&probe_uri, PROBE_SOURCE);
        assert!(
            components
                .iter()
                .all(|(uri, _)| !state.documents.contains(uri)),
            "generated components must remain closed editor documents",
        );

        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(tsgo_path),
            working_dir: Some(project.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.expect("tsgo session");

        let query_offset = PROBE_SOURCE
            .rfind("sharedLabel")
            .expect("probe call reference")
            + 1;
        let ctx = IdeContext::new(&state, &probe_uri, query_offset).expect("context");
        let references =
            ReferencesService::references_with_corsa(&ctx, true, Some(Arc::clone(&bridge)))
                .await
                .expect("project references");
        bridge.shutdown().await.expect("shutdown");

        let generated_references = components
            .iter()
            .map(|(uri, source)| {
                let hits = references
                    .iter()
                    .filter(|location| location.uri == *uri)
                    .collect::<Vec<_>>();
                let expected = if source.contains("v-bind(sharedLabel)") {
                    3
                } else {
                    2
                };
                assert_eq!(
                    hits.len(),
                    expected,
                    "each closed component import and call, plus the closed style binding fixture, must be searched: {references:#?}",
                );
                assert!(
                    hits.iter()
                        .all(|location| authored_text(source, location) == "sharedLabel"),
                    "generated references must map onto authored SFC ranges: {hits:#?}",
                );
                hits.len()
            })
            .sum::<usize>();
        assert_eq!(generated_references, COMPONENT_COUNT * 2 + 1);
        assert_eq!(
            references
                .iter()
                .filter(|location| location.uri == probe_uri)
                .count(),
            2,
            "the probe import and call must remain present: {references:#?}",
        );
        let shared_uri = Url::from_file_path(shared_path).expect("shared uri");
        let shared_hits = references
            .iter()
            .filter(|location| location.uri == shared_uri)
            .collect::<Vec<_>>();
        assert_eq!(
            shared_hits.len(),
            1,
            "includeDeclaration must retain the shared export: {references:#?}",
        );
        assert_eq!(authored_text(shared_source, shared_hits[0]), "sharedLabel");
        assert_eq!(
            references.len(),
            COMPONENT_COUNT * 2 + 4,
            "exact project reference set",
        );
        assert!(
            references
                .iter()
                .all(|location| location.uri != outside_uri),
            "files excluded by tsconfig must stay outside the project search: {references:#?}",
        );
        assert!(
            references
                .iter()
                .all(|location| !location.uri.path().ends_with(".vue.ts")),
            "canonical mirror URIs must never leak: {references:#?}",
        );
    });
}
