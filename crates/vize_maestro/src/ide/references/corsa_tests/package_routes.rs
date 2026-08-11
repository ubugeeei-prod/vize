use super::*;

#[test]
fn package_shadow_references_identity_map_an_authored_typescript_barrel() {
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
            r#"{"name":"@scope/ui","exports":{".":"./index.ts"}}"#,
        )
        .unwrap();
        let package_source = "export { default } from './Entry.vue'\nexport const shared = 1\n";
        let package_path = package.join("index.ts");
        fs::write(&package_path, package_source).unwrap();
        fs::write(
            package.join("Entry.vue"),
            "<script setup lang=\"ts\">defineProps<{ label: string }>()</script>\n",
        )
        .unwrap();
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
        state.documents.open(
            host_uri.clone(),
            host_source.to_string(),
            1,
            "vue".to_string(),
        );
        state.update_virtual_docs(&host_uri, host_source);
        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(tsgo_path),
            working_dir: Some(project.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.unwrap();

        let offset = host_source.find("shared\nvoid").unwrap() + 1;
        let ctx = IdeContext::new(&state, &host_uri, offset).unwrap();
        let document =
            crate::ide::corsa_support::open_canonical_virtual_project_document(&ctx, &bridge)
                .await
                .expect("canonical project document");
        let identities = crate::ide::corsa_support::materialized_semantic_positions(
            &document,
            &package_uri,
            package_source.find("shared =").unwrap() + 1,
        );
        assert!(
            !identities.is_empty(),
            "authored TypeScript barrel identity was not indexed"
        );
        assert!(
            identities.iter().all(|identity| {
                identity.request_uri != document.request_uri
                    && document.materialized_sources.iter().any(|source| {
                        same_file(&source.source_uri, &package_uri)
                            && source.request_uri == identity.request_uri
                    })
            }),
            "package semantic fan-out used an unrelated host identity: {identities:#?}"
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
            locations
                .iter()
                .any(|location| same_file(&location.uri, &package_uri)),
            "authored package declaration missing: {locations:#?}"
        );
        assert!(
            locations
                .iter()
                .any(|location| same_file(&location.uri, &host_uri)),
            "authored importer references missing: {locations:#?}"
        );
        assert!(
            locations.iter().all(|location| {
                !location.uri.path().contains(".vize")
                    && (same_file(&location.uri, &package_uri)
                        || same_file(&location.uri, &host_uri))
            }),
            "package shadow URI leaked: {locations:#?}"
        );
        for location in &locations {
            let source = if same_file(&location.uri, &package_uri) {
                package_source
            } else {
                host_source
            };
            assert_eq!(authored_text(source, location), "shared");
        }
    });
}

fn same_file(left: &Url, right: &Url) -> bool {
    left.to_file_path()
        .ok()
        .and_then(|path| path.canonicalize().ok())
        == right
            .to_file_path()
            .ok()
            .and_then(|path| path.canonicalize().ok())
}
