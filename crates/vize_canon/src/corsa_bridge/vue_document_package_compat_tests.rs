//! Generation settings must be identical for an authored host and every Vue
//! SFC reached through its importer-scoped package mirror.
#![cfg(unix)]

use std::path::{Path, PathBuf};

use super::vue_document::{
    CorsaProjectEnvironment, CorsaVueVirtualDocumentOptions,
    build_vue_virtual_project_with_overlays_and_options_and_package_routes,
};

#[test]
fn package_mirror_preserves_options_class_and_nuxt_page_meta_generation() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    let host = app.join("src/Host.vue");
    let package = app.join("node_modules/@scope/compat");
    std::fs::create_dir_all(host.parent().unwrap()).unwrap();
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        app.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler","allowArbitraryExtensions":true}}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("package.json"),
        r##"{
  "name": "@scope/compat",
  "exports": {
    "./options": "./Options.vue",
    "./class": "./Class.vue",
    "./page": "./Page.vue"
  }
}"##,
    )
    .unwrap();
    write(
        &package.join("Options.vue"),
        r#"<script lang="ts">
export default {
  data() { return { count: 1 } },
  methods: { increment() { this.count++ } }
}
</script>
<template><button @click="increment">{{ count }}</button></template>
"#,
    );
    write(
        &package.join("Class.vue"),
        r#"<script lang="ts">
import { Vue } from 'vue-class-component'
export default class Widget extends Vue {
  greeting = 'hello'
}
</script>
<template>{{ greeting }}</template>
"#,
    );
    write(
        &package.join("Page.vue"),
        r#"<script setup lang="ts">
definePageMeta({ title: 'Importer scoped SEO' })
</script>
<template><main>page</main></template>
"#,
    );
    let nuxt_types = app.join(".nuxt/types/page-meta.d.ts");
    write(
        &nuxt_types,
        "declare function definePageMeta(meta: { title: string }): void;\n",
    );
    let source = r#"<script setup lang="ts">
import OptionsWidget from '@scope/compat/options'
import ClassWidget from '@scope/compat/class'
import PageWidget from '@scope/compat/page'
void OptionsWidget; void ClassWidget; void PageWidget
</script>
"#;
    let mut virtual_options = crate::virtual_ts::VirtualTsOptions::default();
    virtual_options.reference_paths = vec![nuxt_types.to_string_lossy().into_owned().into()];
    let project = build_vue_virtual_project_with_overlays_and_options_and_package_routes(
        &host,
        source,
        CorsaVueVirtualDocumentOptions {
            options_api: true,
            ..Default::default()
        },
        &[],
        CorsaProjectEnvironment {
            virtual_ts_options: &virtual_options,
            package_routes: &crate::PackageRouteResolver::default(),
            project_root: Some(&app),
            tsconfig_path: None,
            editor_session: super::editor_session::fallback_editor_session(),
        },
    )
    .unwrap();

    let options = mapped_code(&project, &package.join("Options.vue"));
    assert!(
        options.contains("type __VizeOptionsInstance<T>"),
        "{options}"
    );
    let class = mapped_code(&project, &package.join("Class.vue"));
    assert!(class.contains("class Widget extends Vue"), "{class}");
    let page = mapped_code(&project, &package.join("Page.vue"));
    assert!(page.contains("definePageMeta({ title: 'Importer scoped SEO' })"));
    assert!(
        page.contains(&nuxt_types.to_string_lossy().replace('\\', "/")),
        "Nuxt reference path missing from package page:\n{page}"
    );
}

fn mapped_code<'a>(
    project: &'a super::vue_document::CorsaVueVirtualProject,
    source: &Path,
) -> &'a str {
    let source = vize_carton::path::canonicalize_non_verbatim(source);
    project
        .host
        .materialized_sources
        .iter()
        .find(|document| document.source_path == source && document.mapping_kind.is_mappable())
        .map(|document| document.code.as_str())
        .expect("mapped package SFC")
}

fn write(path: &PathBuf, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}
