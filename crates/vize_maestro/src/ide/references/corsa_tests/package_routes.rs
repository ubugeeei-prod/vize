use super::*;

#[test]
fn package_shadow_references_map_only_to_authored_vue_uris() {
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
        assert_eq!(state.open_importers(&package_uri), vec![host_uri.clone()]);
        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(tsgo_path),
            working_dir: Some(project.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.unwrap();

        let offset = package_source.find("shared =").unwrap() + 1;
        let ctx = IdeContext::new(&state, &package_uri, offset).unwrap();
        let document =
            crate::ide::corsa_support::open_canonical_virtual_project_document(&ctx, &bridge)
                .await
                .expect("canonical project document");
        let identities = crate::ide::corsa_support::materialized_semantic_positions(
            &document,
            &package_uri,
            offset,
        );
        assert!(
            !identities.is_empty(),
            "package source identity was not indexed"
        );
        let mut raw_identity_references = Vec::new();
        for identity in &identities {
            raw_identity_references.extend(
                bridge
                    .references(
                        &identity.request_uri,
                        identity.line,
                        identity.character,
                        true,
                    )
                    .await
                    .unwrap(),
            );
        }
        assert!(
            raw_identity_references
                .iter()
                .any(|location| location.uri.contains("Host.vue")),
            "native shadow query did not reach importer: identities={identities:#?}, refs={raw_identity_references:#?}"
        );
        let mapped_identity_references = crate::ide::corsa_support::map_canonical_corsa_locations(
            &ctx,
            &document,
            raw_identity_references.clone(),
        );
        assert!(
            mapped_identity_references
                .iter()
                .any(|location| location.uri == host_uri),
            "shadow reference did not map to authored importer: raw={raw_identity_references:#?}, mapped={mapped_identity_references:#?}"
        );
        let locations =
            ReferencesService::references_with_corsa(&ctx, true, Some(Arc::clone(&bridge)))
                .await
                .expect("package references");
        bridge.shutdown().await.unwrap();

        assert!(
            locations.iter().any(|location| location.uri == package_uri),
            "authored package declaration missing: {locations:#?}"
        );
        assert!(
            locations.iter().any(|location| location.uri == host_uri),
            "authored importer references missing: {locations:#?}"
        );
        assert!(
            locations.iter().all(|location| {
                !location.uri.path().contains(".vize")
                    && (location.uri == package_uri || location.uri == host_uri)
            }),
            "package shadow URI leaked: {locations:#?}"
        );
        for location in &locations {
            let source = if location.uri == package_uri {
                package_source
            } else {
                host_source
            };
            assert_eq!(authored_text(source, location), "shared");
        }
    });
}
