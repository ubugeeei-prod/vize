use super::*;

#[test]
fn package_shadow_rename_edits_only_authored_vue_files() {
    crate::runtime::block_on(async {
        let Some(tsgo_path) = resolve_tsgo_binary() else {
            return;
        };
        let project = tempfile::TempDir::new().expect("temp project");
        let src = project.path().join("src");
        let package = project.path().join("node_modules/@scope/ui");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&package).unwrap();
        fs::write(
            project.path().join("tsconfig.json"),
            r#"{"compilerOptions":{"strict":true,"moduleResolution":"bundler","allowArbitraryExtensions":true},"include":["src/**/*"]}"#,
        )
        .unwrap();
        fs::write(
            package.join("package.json"),
            r#"{"name":"@scope/ui","exports":{".":"./Entry.vue"}}"#,
        )
        .unwrap();
        let package_source = r#"<script lang="ts">
export const shared = 1
export default {}
</script>
<template>{{ shared }}</template>
"#;
        let package_path = package.join("Entry.vue");
        fs::write(&package_path, package_source).unwrap();
        let host_source = r#"<script setup lang="ts">
import { shared } from '@scope/ui'
const value = shared
void value
</script>
<template>{{ shared }}</template>
"#;
        let host_path = src.join("Host.vue");
        fs::write(&host_path, host_source).unwrap();
        let package_uri = Url::from_file_path(&package_path).unwrap();
        let host_uri = Url::from_file_path(&host_path).unwrap();
        let state = ServerState::new();
        state.set_workspace_root(project.path().to_path_buf());
        for (uri, source) in [(&package_uri, package_source), (&host_uri, host_source)] {
            state
                .documents
                .open(uri.clone(), source.to_string(), 1, "vue".to_string());
            state.update_virtual_docs(uri, source);
        }
        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(tsgo_path),
            working_dir: Some(project.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.unwrap();

        let offset = package_source.find("shared =").unwrap() + 1;
        let ctx = IdeContext::new(&state, &package_uri, offset).unwrap();
        let prepared = RenameService::prepare_rename_with_corsa(&ctx, Some(Arc::clone(&bridge)))
            .await
            .expect("package prepare rename");
        assert_eq!(
            authored_text(package_source, prepare_range(prepared)),
            "shared"
        );
        let edit = RenameService::rename_with_corsa(&ctx, "renamed", Some(Arc::clone(&bridge)))
            .await
            .expect("package rename");
        bridge.shutdown().await.unwrap();

        let changes = edit.changes.expect("plain authored changes");
        assert_eq!(changes.len(), 2, "only package and importer: {changes:#?}");
        for (uri, source) in [(&package_uri, package_source), (&host_uri, host_source)] {
            let edits = changes.get(uri).expect("authored package rename edits");
            assert!(!edits.is_empty());
            assert!(edits.iter().all(|edit| {
                edit.new_text == "renamed" && authored_text(source, edit.range) == "shared"
            }));
        }
        assert!(
            changes
                .keys()
                .all(|uri| { !uri.path().contains(".vize") && !uri.path().ends_with(".vue.ts") })
        );
    });
}
