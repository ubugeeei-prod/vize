//! Pure typed packages stay on TypeScript's native path; only Vue closures
//! need Canon's importer-scoped package shadow.
#![cfg(unix)]

use super::vue_document::{
    CorsaProjectEnvironment, CorsaVueVirtualDocumentOptions, build_vue_virtual_project,
    build_vue_virtual_project_with_overlays_and_options_and_package_routes,
};

#[test]
fn editor_does_not_materialize_a_shadow_for_a_pure_declaration_package() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    let host = app.join("src/Host.vue");
    let package = app.join("node_modules/@scope/types");
    std::fs::create_dir_all(host.parent().unwrap()).unwrap();
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        app.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler"}}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@scope/types","exports":{".":{"types":"./index.d.ts"}}}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("index.d.ts"),
        "export interface Settings { enabled: boolean }\n",
    )
    .unwrap();
    let source = r#"<script setup lang="ts">
import type { Settings } from '@scope/types'
const settings: Settings = { enabled: true }
void settings
</script>
"#;

    let project =
        build_vue_virtual_project(&host, source, CorsaVueVirtualDocumentOptions::default())
            .unwrap();

    assert!(project.session_project_root.is_none());
    assert!(project.host.materialized_sources.is_empty());
    assert!(project.host.code.contains("from '@scope/types'"));
}

#[test]
fn editor_uses_one_vue_dialect_for_the_host_and_package_dependency() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    let host = app.join("src/Host.vue");
    let package = app.join("node_modules/@scope/ui");
    let component = package.join("Component.vue");
    std::fs::create_dir_all(host.parent().unwrap()).unwrap();
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(
        app.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler","allowArbitraryExtensions":true}}"#,
    )
    .unwrap();
    std::fs::write(
        package.join("package.json"),
        r#"{"name":"@scope/ui","exports":{".":"./Component.vue"}}"#,
    )
    .unwrap();
    let sfc = r#"<script setup lang="ts">
import { ref } from 'vue'
const count = ref(0)
</script>
<template>{{ count }}</template>
"#;
    std::fs::write(&component, sfc).unwrap();
    let host_source = r#"<script setup lang="ts">
import Widget from '@scope/ui'
import { ref } from 'vue'
const count = ref(0)
void Widget
</script>
<template>{{ count }}</template>
"#;

    for dialect in vize_carton::config::VueVersion::ALL {
        let project = build_vue_virtual_project_with_overlays_and_options_and_package_routes(
            &host,
            host_source,
            CorsaVueVirtualDocumentOptions {
                dialect,
                ..Default::default()
            },
            &[],
            CorsaProjectEnvironment {
                virtual_ts_options: &Default::default(),
                package_routes: &crate::PackageRouteResolver::default(),
                project_root: Some(&app),
                tsconfig_path: None,
                editor_session: super::editor_session::fallback_editor_session(),
            },
        )
        .unwrap();
        let canonical_component = vize_carton::path::canonicalize_non_verbatim(&component);
        let dependency = project
            .host
            .materialized_sources
            .iter()
            .find(|source| {
                source.source_path == canonical_component && source.mapping_kind.is_mappable()
            })
            .expect("package Vue source");
        let host_uses_vue2_instance = project.host.code.contains("__VizeVue2ComponentInstance");
        let dependency_uses_vue2_instance = dependency.code.contains("__VizeVue2ComponentInstance");
        assert_eq!(
            host_uses_vue2_instance, dependency_uses_vue2_instance,
            "host and dependency diverged for {dialect:?}"
        );
        if matches!(
            dialect,
            vize_carton::config::VueVersion::V2 | vize_carton::config::VueVersion::V2_7
        ) {
            assert!(
                dependency_uses_vue2_instance,
                "dependency dialect {dialect:?}"
            );
        }
    }
}
