//! Importer-scoped package shadows in editor sessions (#4002).
#![cfg(unix)]

use std::path::{Path, PathBuf};

use super::vue_document::{CorsaVueVirtualDocumentOptions, build_vue_virtual_project};

#[path = "vue_document_package_tests/union.rs"]
mod union;

const TSCONFIG: &str = r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler","allowArbitraryExtensions":true,"customConditions":["editor"]}}"#;

#[test]
fn editor_route_is_importer_scoped_and_materializes_native_shadow_topology() {
    let fixture = package_fixture("alpha", "alpha", "string");
    let source = host_source("alpha", "'ok'");
    let project = build(&fixture.host, &source);

    assert!(project.host.code.contains("from '@scope/ui'"));
    assert!(!project.host.code.contains(".vize-package-routes"));
    let host = crate::file_uri::file_uri_to_path(&project.host.request_uri).unwrap();
    assert_ne!(host, fixture.host.with_extension("vue.ts"));
    let shadow = shadow_root(&host);
    assert_eq!(
        std::fs::read_to_string(shadow.join("package.json")).unwrap(),
        package_manifest("alpha", "string")
    );
    assert!(shadow.join("src/Conditional.vue.ts").is_file());
    assert!(shadow.join("src/Conditional.d.vue.ts").is_file());
    assert!(shadow.join("src/Internal.d.vue.ts").is_file());
}

#[test]
fn duplicate_package_versions_materialize_distinct_editor_identities() {
    let root = tempfile::tempdir().unwrap();
    let alpha = app_fixture(root.path(), "alpha", "alpha", "string");
    let bravo = app_fixture(root.path(), "bravo", "bravo", "number");

    let alpha_project = build(&alpha, &host_source("alpha", "'ok'"));
    let bravo_project = build(&bravo, &host_source("bravo", "1"));
    let alpha_shadow = request_path(&alpha_project);
    let bravo_shadow = request_path(&bravo_project);

    assert_ne!(alpha_shadow, bravo_shadow);
    assert!(
        std::fs::read_to_string(selected_companion(&alpha_shadow))
            .unwrap()
            .contains("alpha")
    );
    assert!(
        std::fs::read_to_string(selected_companion(&bravo_shadow))
            .unwrap()
            .contains("bravo")
    );
    assert!(alpha_project.host.code.contains("from '@scope/ui'"));
    assert!(bravo_project.host.code.contains("from '@scope/ui'"));
}

#[test]
fn corsa_editor_queries_observe_the_importer_local_package_shadow() {
    let Some(corsa_path) = std::env::var_os("CORSA_PATH").map(PathBuf::from) else {
        return;
    };
    if !corsa_path.is_file() {
        return;
    }
    let fixture = package_fixture("queries", "alpha", "string");
    let source = host_source("bravo", "1");
    std::fs::write(&fixture.host, &source).unwrap();
    let app = fixture.host.ancestors().nth(2).unwrap().to_path_buf();
    install_runtime_stubs(&app);
    let bridge = super::CorsaBridge::with_config(super::CorsaBridgeConfig {
        corsa_path: Some(corsa_path),
        working_dir: Some(app),
        timeout_ms: 30_000,
        ..Default::default()
    });

    corsa::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        let document = bridge
            .open_vue_virtual_document(
                &fixture.host,
                &source,
                CorsaVueVirtualDocumentOptions::default(),
            )
            .await
            .unwrap();
        let diagnostics = bridge.get_diagnostics(&document.request_uri).await.unwrap();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code.as_ref().is_some_and(|code| code == 2353)),
            "{diagnostics:#?}"
        );

        let widget = document.code.find("Widget").unwrap();
        let (line, character) = line_column(&document.code, widget);
        assert!(
            bridge
                .hover(&document.request_uri, line, character)
                .await
                .unwrap()
                .is_some()
        );
        let definitions = bridge
            .definition(&document.request_uri, line, character)
            .await
            .unwrap();
        assert!(
            definitions
                .iter()
                .filter_map(|location| crate::file_uri::file_uri_to_path(&location.uri))
                .any(|path| path.to_string_lossy().contains("node_modules/@scope/ui")),
            "{definitions:#?}"
        );
        assert!(
            !bridge
                .references(&document.request_uri, line, character, true)
                .await
                .unwrap()
                .is_empty()
        );
        assert!(
            bridge
                .prepare_rename(&document.request_uri, line, character)
                .await
                .unwrap()
                .is_some()
        );
        let props_object = document.code.find("{ bravo: 1 }").unwrap() + 2;
        let (completion_line, completion_character) = line_column(&document.code, props_object);
        let completions = bridge
            .completion(&document.request_uri, completion_line, completion_character)
            .await
            .unwrap();
        assert!(
            completions.iter().any(|item| item.label == "alpha"),
            "{completions:#?}"
        );
        bridge.shutdown().await.unwrap();
    });
}

struct PackageFixture {
    _root: tempfile::TempDir,
    host: PathBuf,
    package: PathBuf,
}

fn package_fixture(package_dir: &str, prop: &str, ty: &str) -> PackageFixture {
    let root = tempfile::tempdir().unwrap();
    let host = app_fixture(root.path(), package_dir, prop, ty);
    PackageFixture {
        package: root.path().join(package_dir).join("node_modules/@scope/ui"),
        _root: root,
        host,
    }
}

fn app_fixture(root: &Path, app: &str, prop: &str, ty: &str) -> PathBuf {
    let app_root = root.join(app);
    let host = app_root.join("src/Host.vue");
    let package = app_root.join("node_modules/@scope/ui");
    std::fs::create_dir_all(host.parent().unwrap()).unwrap();
    std::fs::create_dir_all(package.join("src")).unwrap();
    std::fs::write(app_root.join("tsconfig.json"), TSCONFIG).unwrap();
    std::fs::write(package.join("package.json"), package_manifest(prop, ty)).unwrap();
    std::fs::write(
        package.join("src/Internal.vue"),
        format!("<script setup lang=\"ts\">defineProps<{{ {prop}: {ty} }}>()</script>\n"),
    )
    .unwrap();
    std::fs::write(
        package.join("src/Conditional.vue"),
        format!(
            "<script setup lang=\"ts\">import Internal from '#internal'; void Internal; defineProps<{{ {prop}: {ty} }}>()</script>\n"
        ),
    )
    .unwrap();
    std::fs::write(
        package.join("src/Fallback.vue"),
        "<script setup lang=\"ts\">defineProps<{ fallback: Date }>()</script>\n",
    )
    .unwrap();
    host
}

fn package_manifest(prop: &str, ty: &str) -> String {
    format!(
        "{{\n  \"name\": \"@scope/ui\",\n  \"version\": \"{prop}-{ty}\",\n  \"exports\": {{ \".\": {{ \"editor\": \"./src/Conditional.vue\", \"types\": \"./src/Fallback.vue\" }} }},\n  \"imports\": {{ \"#internal\": \"./src/Internal.vue\" }}\n}}\n"
    )
}

fn host_source(prop: &str, value: &str) -> String {
    format!(
        "<script setup lang=\"ts\">\nimport Widget from '@scope/ui'\ntype Props = InstanceType<typeof Widget>['$props']\nconst props: Props = {{ {prop}: {value} }}\nvoid props\n</script>\n"
    )
}

fn build(host: &Path, source: &str) -> super::vue_document::CorsaVueVirtualProject {
    build_vue_virtual_project(host, source, CorsaVueVirtualDocumentOptions::default()).unwrap()
}

fn line_column(source: &str, offset: usize) -> (u32, u32) {
    let before = &source[..offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() as u32;
    let character = before
        .rsplit_once('\n')
        .map_or(before.len(), |(_, tail)| tail.len()) as u32;
    (line, character)
}

fn request_path(project: &super::vue_document::CorsaVueVirtualProject) -> PathBuf {
    crate::file_uri::file_uri_to_path(&project.host.request_uri).unwrap()
}

fn shadow_root(host: &Path) -> PathBuf {
    host.parent().unwrap().join("node_modules/@scope/ui")
}

fn selected_companion(host: &Path) -> PathBuf {
    shadow_root(host).join("src/Conditional.vue.ts")
}

fn install_runtime_stubs(project_root: &Path) {
    let node_modules = project_root.join("node_modules");
    crate::batch::write_vue_facade(&node_modules).unwrap();
    let runtime_dom = node_modules.join("@vue/runtime-dom");
    std::fs::create_dir_all(&runtime_dom).unwrap();
    std::fs::write(
        runtime_dom.join("package.json"),
        "{\"name\":\"@vue/runtime-dom\",\"types\":\"index.d.ts\"}\n",
    )
    .unwrap();
    std::fs::write(
        runtime_dom.join("index.d.ts"),
        crate::batch::VUE_RUNTIME_DOM_STUB_TYPES,
    )
    .unwrap();

    let vite = node_modules.join("vite");
    std::fs::create_dir_all(&vite).unwrap();
    std::fs::write(
        vite.join("package.json"),
        "{\"name\":\"vite\",\"exports\":{\"./client\":{\"types\":\"./client.d.ts\"}}}\n",
    )
    .unwrap();
    std::fs::write(vite.join("client.d.ts"), "export {};\n").unwrap();
}
