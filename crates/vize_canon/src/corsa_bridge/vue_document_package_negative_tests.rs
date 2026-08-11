//! Native package diagnostics must land on the mirror host, never a proxy.

use std::path::{Path, PathBuf};

use super::vue_document::CorsaVueVirtualDocumentOptions;

#[test]
fn named_only_and_missing_vue_exports_report_on_the_authored_host() {
    let Some(corsa_path) = std::env::var_os("CORSA_PATH").map(PathBuf::from) else {
        return;
    };
    if !corsa_path.is_file() {
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    let host = app.join("src/Host.vue");
    write(
        &app.join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true,"module":"ESNext","moduleResolution":"Bundler","allowArbitraryExtensions":true}}"#,
    );
    let named = app.join("node_modules/@scope/named-only");
    write(
        &named.join("package.json"),
        r#"{"name":"@scope/named-only","exports":"./index.ts"}"#,
    );
    write(
        &named.join("index.ts"),
        "export { default as Widget } from './Widget.vue'\n",
    );
    write(
        &named.join("Widget.vue"),
        "<script setup lang=\"ts\">defineProps<{ exact: string }>()</script>\n",
    );
    let missing = app.join("node_modules/@scope/missing-vue");
    write(
        &missing.join("package.json"),
        r#"{"name":"@scope/missing-vue","exports":"./Missing.vue"}"#,
    );
    let source = r#"<script setup lang="ts">
import InvalidDefault from '@scope/named-only'
import Missing from '@scope/missing-vue'
void InvalidDefault
void Missing
</script>
"#;
    write(&host, source);
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
            .open_vue_virtual_document(&host, source, CorsaVueVirtualDocumentOptions::default())
            .await
            .unwrap();
        assert!(document.code.contains("from '@scope/named-only'"));
        assert!(!document.code.contains(".vize-package-routes"));
        let diagnostics = bridge.get_diagnostics(&document.request_uri).await.unwrap();
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic
                    .code
                    .as_ref()
                    .is_some_and(|code| code == 1192 || code == 2613)
            }),
            "named-only default import was hidden: {diagnostics:#?}"
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code.as_ref().is_some_and(|code| code == 2307)),
            "missing Vue export was hidden: {diagnostics:#?}"
        );

        write(
            &missing.join("Missing.vue"),
            "<script setup lang=\"ts\">defineProps<{ created: true }>()</script>\n",
        );
        let created = bridge
            .open_vue_virtual_document(&host, source, CorsaVueVirtualDocumentOptions::default())
            .await
            .unwrap();
        let created_request = crate::file_uri::file_uri_to_path(&created.request_uri).unwrap();
        let created_shadow = created_request
            .parent()
            .unwrap()
            .join("node_modules/@scope/missing-vue");
        assert!(
            created_shadow.join("package.json").is_file(),
            "created package manifest was not materialized at {}",
            created_shadow.display()
        );
        assert!(
            created_shadow.join("Missing.d.vue.ts").is_file(),
            "created package target was not materialized at {}",
            created_shadow.display()
        );
        let created_diagnostics = bridge.get_diagnostics(&created.request_uri).await.unwrap();
        assert!(
            !created_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code.as_ref().is_some_and(|code| code == 2307)),
            "creating the missing package target did not refresh Corsa: {created_diagnostics:#?}"
        );

        std::fs::remove_file(missing.join("Missing.vue")).unwrap();
        let deleted = bridge
            .open_vue_virtual_document(&host, source, CorsaVueVirtualDocumentOptions::default())
            .await
            .unwrap();
        let deleted_diagnostics = bridge.get_diagnostics(&deleted.request_uri).await.unwrap();
        assert!(
            deleted_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code.as_ref().is_some_and(|code| code == 2307)),
            "deleting the package target did not refresh Corsa: {deleted_diagnostics:#?}"
        );
        bridge.shutdown().await.unwrap();
    });
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

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
