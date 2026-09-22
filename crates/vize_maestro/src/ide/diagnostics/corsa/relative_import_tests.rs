use super::collect_virtual::{
    CorsaDocument, assemble_corsa_diagnostics, fetch_finished_diagnostics,
};
use crate::DiagnosticService;
use tower_lsp::lsp_types::Url;
use vize_canon::{CorsaBridge, CorsaBridgeConfig, CorsaVueVirtualDocumentOptions};

#[test]
fn editor_relative_vue_imports_resolve_existing_siblings() {
    let Some(corsa_path) = resolve_test_tsgo_binary() else {
        return;
    };

    let project = tempfile::TempDir::new().expect("temp project");
    let root = project.path();
    let src = root.join("src");
    let layout = src.join("layout");
    let logo = src.join("logo");
    let tag = src.join("tag");
    std::fs::create_dir_all(&layout).expect("layout dir");
    std::fs::create_dir_all(&logo).expect("logo dir");
    std::fs::create_dir_all(&tag).expect("tag dir");

    std::fs::write(
        root.join("tsconfig.json"),
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
    std::fs::write(
        src.join("vue.d.ts"),
        r#"declare module "vue" {
  export interface Ref<T = unknown, _Raw = T> { value: T }
  export interface ComputedRef<T = unknown> extends Ref<T> {}
  export interface ShallowRef<T = unknown, _Raw = T> extends Ref<T> {}
  export interface ComponentPublicInstance {
    $attrs: Record<string, unknown>;
    $slots: Record<string, (...args: any[]) => any>;
    $refs: Record<string, any>;
    $emit: (...args: any[]) => void;
  }
  export type DefineComponent<P = any, B = any, D = any, C = any, M = any, Mixins = any, Extends = any, E = any> = {
    new (): { $props: P };
  };
}
"#,
    )
    .expect("vue shim");

    std::fs::write(
        logo.join("MfMatesLogo.vue"),
        r#"<script setup lang="ts">
defineProps<{ label?: string }>();
</script>
<template><svg aria-hidden="true" /></template>
"#,
    )
    .expect("logo sfc");
    std::fs::write(
        tag.join("MfTag.vue"),
        r#"<script setup lang="ts">
defineProps<{ tone?: "neutral" | "accent" }>();
</script>
<template><span /></template>
"#,
    )
    .expect("tag sfc");

    let host_path = layout.join("MfPageShell.vue");
    let host_content = r#"<script setup lang="ts">
import MfMatesLogo from "../logo/MfMatesLogo.vue";
import MfTag from "../tag/MfTag.vue";

const {
  brandAriaLabel = "Go to Mates Internal top",
  eyebrow,
  footerCopy = null,
  lead,
  meta = [],
  title,
} = defineProps<{
  brandAriaLabel?: string;
  eyebrow?: string;
  footerCopy?: string | null;
  lead?: string;
  meta?: readonly string[];
  title: string;
}>();

defineSlots<{
  actions?: () => unknown;
  brand?: () => unknown;
  default?: () => unknown;
}>();
</script>
<template>
  <header>
    <MfMatesLogo :label="brandAriaLabel" />
    <MfTag v-if="eyebrow" tone="accent">{{ eyebrow }}</MfTag>
    <h1>{{ title }}</h1>
    <p v-if="lead">{{ lead }}</p>
    <small v-if="footerCopy">{{ footerCopy }}</small>
    <span v-for="item in meta" :key="item">{{ item }}</span>
  </header>
</template>
"#;
    std::fs::write(&host_path, host_content).expect("host sfc");

    let host_uri = Url::from_file_path(&host_path).expect("host uri");
    let virtual_result =
        DiagnosticService::generate_virtual_ts(&host_uri, host_content, false, false)
            .expect("virtual ts generated");
    assert!(
        virtual_result.code.contains("../logo/MfMatesLogo.vue.ts")
            && virtual_result.code.contains("../tag/MfTag.vue.ts"),
        "editor virtual TS must rewrite sibling Vue imports to virtual mirrors:\n{}",
        virtual_result.code,
    );
    let bridge = std::sync::Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
        corsa_path: Some(corsa_path),
        working_dir: Some(root.to_path_buf()),
        timeout_ms: 30_000,
        ..Default::default()
    }));

    let diagnostics = crate::runtime::block_on(async {
        if bridge.spawn().await.is_err() {
            return None;
        }
        let result = async {
            let opened = bridge
                .open_vue_virtual_document(
                    &host_path,
                    host_content,
                    CorsaVueVirtualDocumentOptions::default(),
                )
                .await
                .ok()?;
            let host_mirror = opened
                .materialized_sources
                .iter()
                .find(|source| {
                    vize_s0::path::canonicalize_non_verbatim(&source.source_path)
                        == vize_s0::path::canonicalize_non_verbatim(&host_path)
                })
                .expect("host must be materialized beside its sibling dependencies");
            assert_eq!(
                opened.request_uri.as_str(),
                Url::from_file_path(&host_mirror.materialized_path)
                    .unwrap()
                    .as_str()
            );
            assert_eq!(opened.code, host_mirror.code);
            let expected_imports = [("MfMatesLogo", "MfMatesLogo.vue"), ("MfTag", "MfTag.vue")]
                .map(|(name, file)| {
                    let dependency = opened
                        .dependencies
                        .iter()
                        .find(|dependency| {
                            dependency
                                .source_path
                                .file_name()
                                .is_some_and(|candidate| candidate == file)
                        })
                        .unwrap();
                    let path = Url::parse(dependency.request_uri.as_str())
                        .unwrap()
                        .to_file_path()
                        .unwrap();
                    vize_s0::cstr!(
                        "import {name} from \"{}\";",
                        path.to_string_lossy().replace('\\', "/")
                    )
                });
            let actual_imports = opened
                .code
                .lines()
                .filter(|line| {
                    line.starts_with("import MfMatesLogo ") || line.starts_with("import MfTag ")
                })
                .collect::<Vec<_>>();
            assert_eq!(actual_imports, expected_imports);
            let (virtual_uri, virtual_result) =
                DiagnosticService::virtual_ts_result_from_corsa_vue_document(opened);
            let document = CorsaDocument::from_result(virtual_result);
            let finished = fetch_finished_diagnostics(&bridge, &virtual_uri, &document, 0)
                .await
                .ok()?;
            Some(assemble_corsa_diagnostics(
                host_content,
                &[document],
                finished,
            ))
        }
        .await;
        let _ = bridge.shutdown().await;
        result
    });

    let Some(diagnostics) = diagnostics else {
        return;
    };
    assert!(
        diagnostics.is_empty(),
        "existing sibling Vue imports must not produce any editor Corsa diagnostics: {diagnostics:#?}",
    );
    assert!(
        diagnostics.iter().all(|diagnostic| {
            !diagnostic.message.contains("Cannot find module")
                && !diagnostic.message.contains(".vue.ts")
        }),
        "existing .vue imports must not surface .vue.ts module-resolution diagnostics: {diagnostics:#?}",
    );
}

fn resolve_test_tsgo_binary() -> Option<std::path::PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }

    let workspace_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)?;
    let sibling_cache = workspace_root.parent()?.join("corsa-bind/.cache/tsgo");
    if sibling_cache.exists() {
        return Some(sibling_cache);
    }

    vize_s0::corsa_resolver::discover_corsa_in_ancestors(workspace_root)
}
