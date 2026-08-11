//! Effective editor config and inferred-project package routing.

use std::path::{Path, PathBuf};

use super::vue_document::{CorsaVueVirtualDocumentOptions, build_vue_virtual_project};

#[test]
fn inferred_project_without_tsconfig_still_queries_an_importer_scoped_package_mirror() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    let host = app.join("src/Host.vue");
    let package = app.join("node_modules/@scope/ui");
    write(&host, SOURCE);
    write(
        &package.join("package.json"),
        r#"{"name":"@scope/ui","exports":"./Widget.vue"}"#,
    );
    write(
        &package.join("Widget.vue"),
        "<script setup lang=\"ts\">defineProps<{ editorOnly: string }>()</script>\n",
    );

    let project =
        build_vue_virtual_project(&host, SOURCE, CorsaVueVirtualDocumentOptions::default())
            .unwrap();
    let request = crate::file_uri::file_uri_to_path(&project.host.request_uri).unwrap();

    assert_ne!(request, host.with_extension("vue.ts"));
    assert!(
        request
            .parent()
            .unwrap()
            .join("node_modules/@scope/ui/Widget.vue.ts")
            .is_file()
    );
}

#[test]
fn solution_hosts_with_different_effective_configs_use_distinct_editor_projects() {
    let root = tempfile::tempdir().unwrap();
    let app_host = root.path().join("app/App.vue");
    let node_host = root.path().join("node/Node.vue");
    let package = root.path().join("node_modules/@scope/ui");
    write(
        &root.path().join("tsconfig.json"),
        r#"{"files":[],"references":[{"path":"./tsconfig.app.json"},{"path":"./tsconfig.node.json"}]}"#,
    );
    write(
        &root.path().join("tsconfig.app.json"),
        r#"{"compilerOptions":{"moduleResolution":"bundler","customConditions":["app"]},"include":["app/**/*"]}"#,
    );
    write(
        &root.path().join("tsconfig.node.json"),
        r#"{"compilerOptions":{"moduleResolution":"nodenext","module":"nodenext","customConditions":["node"]},"include":["node/**/*"]}"#,
    );
    write(&app_host, SOURCE);
    write(&node_host, SOURCE);
    write(
        &package.join("package.json"),
        r#"{"name":"@scope/ui","exports":{ ".":{"app":"./App.vue","node":"./Node.vue","default":"./App.vue"}}}"#,
    );
    write(
        &package.join("App.vue"),
        "<script setup lang=\"ts\">defineProps<{ editorOnly: string }>()</script>\n",
    );
    write(
        &package.join("Node.vue"),
        "<script setup lang=\"ts\">defineProps<{ nodeOnly: number }>()</script>\n",
    );

    let app =
        build_vue_virtual_project(&app_host, SOURCE, CorsaVueVirtualDocumentOptions::default())
            .unwrap();
    let node = build_vue_virtual_project(
        &node_host,
        SOURCE,
        CorsaVueVirtualDocumentOptions::default(),
    )
    .unwrap();
    let app_root = app.session_project_root.unwrap();
    let node_root = node.session_project_root.unwrap();
    assert_ne!(
        app_root, node_root,
        "incompatible referenced projects must not share one mirror tsconfig"
    );
    let app_config = std::fs::read_to_string(app_root.join("tsconfig.json")).unwrap();
    let node_config = std::fs::read_to_string(node_root.join("tsconfig.json")).unwrap();
    assert!(app_config.contains("\"app\""));
    assert!(!app_config.contains("\"node\""));
    assert!(node_config.contains("\"node\""));
    assert!(!node_config.contains("\"app\""));
}

#[test]
fn explicit_nonstandard_tsconfig_controls_editor_package_conditions() {
    let Some(corsa_path) = std::env::var_os("CORSA_PATH").map(PathBuf::from) else {
        return;
    };
    if !corsa_path.is_file() {
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    let host = app.join("src/Host.vue");
    let package = app.join("node_modules/@scope/ui");
    write(
        &app.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler","allowArbitraryExtensions":true,"customConditions":["base"]}}"#,
    );
    let explicit = app.join("tsconfig.vize.json");
    let base_config = app.join("tsconfig.base.json");
    write(
        &base_config,
        r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler","allowArbitraryExtensions":true,"customConditions":["editor"]}}"#,
    );
    write(&explicit, r#"{"extends":"./tsconfig.base.json"}"#);
    write(&host, SOURCE);
    write(
        &package.join("package.json"),
        r#"{"name":"@scope/ui","exports":{".":{"editor":"./Editor.vue","legacy":"./Legacy.vue","base":"./Base.vue","default":"./Base.vue"}}}"#,
    );
    write(
        &package.join("Editor.vue"),
        "<script setup lang=\"ts\">defineProps<{ editorOnly: string }>()</script>\n",
    );
    write(
        &package.join("Base.vue"),
        "<script setup lang=\"ts\">defineProps<{ baseOnly: number }>()</script>\n",
    );
    write(
        &package.join("Legacy.vue"),
        "<script setup lang=\"ts\">defineProps<{ legacyOnly: boolean }>()</script>\n",
    );
    install_runtime_stubs(&app);
    let bridge = super::CorsaBridge::with_config(super::CorsaBridgeConfig {
        corsa_path: Some(corsa_path),
        working_dir: Some(app),
        tsconfig_path: Some(explicit),
        timeout_ms: 30_000,
        ..Default::default()
    });

    corsa::runtime::block_on(async {
        bridge.spawn().await.unwrap();
        let document = bridge
            .open_vue_virtual_document(&host, SOURCE, CorsaVueVirtualDocumentOptions::default())
            .await
            .unwrap();
        let diagnostics = bridge.get_diagnostics(&document.request_uri).await.unwrap();
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code.as_ref().is_none_or(|code| code != 2353)),
            "nearest tsconfig.json overrode the configured tsconfig: {diagnostics:#?}"
        );

        let modified = std::fs::metadata(&base_config).unwrap().modified().unwrap();
        let legacy_config = std::fs::read_to_string(&base_config)
            .unwrap()
            .replace("editor", "legacy");
        std::fs::write(&base_config, legacy_config).unwrap();
        std::fs::File::options()
            .write(true)
            .open(&base_config)
            .unwrap()
            .set_modified(modified)
            .unwrap();
        let legacy = bridge
            .open_vue_virtual_document(&host, SOURCE, CorsaVueVirtualDocumentOptions::default())
            .await
            .unwrap();
        let legacy_diagnostics = bridge.get_diagnostics(&legacy.request_uri).await.unwrap();
        assert!(
            legacy_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code.as_ref().is_some_and(|code| code == 2353)),
            "same-mtime base tsconfig condition flip reused the stale route: {legacy_diagnostics:#?}"
        );
        bridge.shutdown().await.unwrap();
    });
}

const SOURCE: &str = r#"<script setup lang="ts">
import Widget from '@scope/ui'
type Props = InstanceType<typeof Widget>['$props']
const props: Props = { editorOnly: 'ok' }
void props
</script>
"#;

fn install_runtime_stubs(project_root: &Path) {
    let node_modules = project_root.join("node_modules");
    crate::batch::write_vue_facade(&node_modules).unwrap();
    let runtime_dom = node_modules.join("@vue/runtime-dom");
    write(
        &runtime_dom.join("package.json"),
        r#"{"name":"@vue/runtime-dom","types":"index.d.ts"}"#,
    );
    write(
        &runtime_dom.join("index.d.ts"),
        crate::batch::VUE_RUNTIME_DOM_STUB_TYPES,
    );
    let vite = node_modules.join("vite");
    write(
        &vite.join("package.json"),
        r#"{"name":"vite","exports":{"./client":{"types":"./client.d.ts"}}}"#,
    );
    write(&vite.join("client.d.ts"), "export {};\n");
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}
