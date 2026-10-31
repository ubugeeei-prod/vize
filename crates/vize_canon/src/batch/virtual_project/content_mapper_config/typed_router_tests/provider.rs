//! Source-built and published Router authority through the mapper API.

use std::path::{Path, PathBuf};

use serde_json::json;
use sha2::{Digest, Sha256};

use crate::{
    ContentMapperProjectConfiguration, ContentMapperTransform,
    generate_vue_content_mapper_transform_with_project_context,
    generate_vue_content_mapper_transform_with_project_options,
};

pub(in crate::batch::virtual_project) const PAGE: &str = "<script setup lang=\"ts\">\nimport { definePage } from 'vue-router/experimental'\nconst props = defineProps<{ extra?: string }>()\ndefinePage({ props: true })\nvoid props\n</script>\n<template>{{ id }}</template>";
const CONFIG: &str = r#"{"compilerOptions":{"moduleResolution":"Bundler","rootDir":"."},"vueCompilerOptions":{"plugins":["vue-router/volar/sfc-typed-router"]}}"#;

pub(in crate::batch::virtual_project) fn install(root: &Path, kind: &str) {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/typechecker/experimental-router-project-capability");
    let authority: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture.join("artifacts.json")).unwrap()).unwrap();
    let provider = authority["providers"]
        .as_array()
        .unwrap()
        .iter()
        .find(|provider| provider["kind"] == kind)
        .unwrap();
    assert_eq!(provider["version"], "5.4.0");
    let package = root.join("node_modules/vue-router");
    if package.exists() {
        std::fs::remove_dir_all(&package).unwrap();
    }
    for entry in provider["files"].as_array().unwrap() {
        let path = entry["path"].as_str().unwrap();
        let bytes = std::fs::read(fixture.join(kind).join(vize_l0::cstr!("{path}.txt"))).unwrap();
        assert_eq!(bytes.len() as u64, entry["bytes"].as_u64().unwrap());
        assert_eq!(
            super::super::super::identity::encode_digest(Sha256::digest(&bytes)),
            entry["sha256"].as_str().unwrap()
        );
        let target = package.join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, bytes).unwrap();
    }
}

fn load(root: &Path, resolution: serde_json::Value) -> ContentMapperProjectConfiguration {
    ContentMapperProjectConfiguration::load_with_compiler_options(
        &root.join("tsconfig.json"),
        json!({"rootDir":root,"moduleResolution":resolution})
            .as_object()
            .unwrap(),
    )
    .unwrap()
    .0
}

fn project(kind: &str) -> (tempfile::TempDir, PathBuf) {
    let guard = tempfile::tempdir().unwrap();
    let root = vize_carton::path::canonicalize_non_verbatim(guard.path());
    std::fs::write(root.join("tsconfig.json"), CONFIG).unwrap();
    std::fs::write(
        root.join("package.json"),
        r#"{"name":"mapper-context-fixture","private":true}"#,
    )
    .unwrap();
    install(&root, kind);
    (guard, root)
}

fn transform(root: &Path, config: &ContentMapperProjectConfiguration) -> ContentMapperTransform {
    generate_vue_content_mapper_transform_with_project_context(
        &root.join("src/pages/[id].vue"),
        PAGE,
        config.transform_options,
        &config.project_context,
    )
    .unwrap()
}

fn expected_watch(root: &Path, entry: &str) -> Vec<vize_l0::String> {
    let mut paths = [
        "tsconfig.json".to_owned(),
        "package.json".to_owned(),
        "node_modules/vue-router".to_owned(),
        "node_modules/vue-router/package.json".to_owned(),
        "node_modules/vue-router/dist/experimental/index.d.ts".to_owned(),
        format!("node_modules/vue-router/dist/{entry}.ts"),
        format!("node_modules/vue-router/dist/{entry}.tsx"),
        format!("node_modules/vue-router/dist/{entry}.d.ts"),
    ]
    .map(|relative| root.join(relative).to_string_lossy().into_owned().into())
    .to_vec();
    paths.sort();
    paths
}

#[test]
fn both_actual_same_version_providers_are_observed_without_granting_native_authority() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<crate::ContentMapperProjectContext>();
    let (_preview_guard, preview) = project("preview");
    let (_published_guard, published) = project("published");
    let admitted = load(&preview, json!(100));
    let refused = load(&published, json!("Bundler"));
    let enabled = transform(&preview, &admitted);
    let disabled = transform(&published, &refused);
    assert_eq!(
        admitted
            .project_context
            .config()
            .map(|config| config.route_props),
        Some(false)
    );
    assert_eq!(
        refused
            .project_context
            .config()
            .map(|config| config.route_props),
        Some(false)
    );
    assert_eq!(
        admitted.watched_files,
        expected_watch(&preview, "index-BA_Jc-EQ")
    );
    assert_eq!(
        refused.watched_files,
        expected_watch(&published, "index-u4XkZVDG")
    );
    let legacy = generate_vue_content_mapper_transform_with_project_options(
        &preview.join("src/pages/[id].vue"),
        PAGE,
        admitted.transform_options,
        Some(&preview),
    )
    .unwrap();
    let published_legacy = generate_vue_content_mapper_transform_with_project_options(
        &published.join("src/pages/[id].vue"),
        PAGE,
        refused.transform_options,
        Some(&published),
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(&disabled).unwrap(),
        serde_json::to_value(published_legacy).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&enabled).unwrap(),
        serde_json::to_value(&legacy).unwrap()
    );
    assert_eq!(
        serde_json::to_value(transform(&preview, &admitted)).unwrap(),
        serde_json::to_value(&enabled).unwrap()
    );
    for resolution in [
        json!(3),
        json!(99),
        json!(100),
        json!("Node16"),
        json!("NodeNext"),
        json!("Bundler"),
    ] {
        assert_eq!(
            serde_json::to_value(transform(&preview, &load(&preview, resolution))).unwrap(),
            serde_json::to_value(&enabled).unwrap(),
        );
    }
    for resolution in [
        json!(2),
        json!(777),
        json!("unknown"),
        json!(-1),
        json!(100.5),
        json!(true),
        json!(null),
    ] {
        let config = load(&preview, resolution);
        assert_eq!(
            config.watched_files,
            vec![preview.join("tsconfig.json").to_string_lossy().into_owned()]
        );
        assert_eq!(
            serde_json::to_value(transform(&preview, &config)).unwrap(),
            serde_json::to_value(&legacy).unwrap(),
        );
    }
}

#[test]
fn every_unproven_file_keeps_the_complete_legacy_file_only_packet() {
    let (_guard, root) = project("preview");
    let config = load(&root, json!(100));
    for file in [
        root.join("src/pages/[id].vue"),
        root.join("nested/Page.vue"),
        root.join("../outside/Page.vue"),
    ] {
        let actual = generate_vue_content_mapper_transform_with_project_context(
            &file,
            PAGE,
            config.transform_options,
            &config.project_context,
        )
        .unwrap();
        let legacy = generate_vue_content_mapper_transform_with_project_options(
            &file,
            PAGE,
            config.transform_options,
            Some(&root),
        )
        .unwrap();
        assert_eq!(
            serde_json::to_value(actual).unwrap(),
            serde_json::to_value(legacy).unwrap()
        );
    }
}

#[test]
fn reopening_tracks_provider_bytes_reexports_removal_and_configuration_changes() {
    let (_guard, root) = project("preview");
    let before = load(&root, json!(100));
    let output = serde_json::to_value(transform(&root, &before)).unwrap();
    let entry = root.join("node_modules/vue-router/dist/experimental/index.d.ts");
    let original = std::fs::read(&entry).unwrap();
    std::fs::remove_file(&entry).unwrap();
    // Captured authority is immutable; a transform does not re-read admission inputs.
    assert_eq!(
        serde_json::to_value(transform(&root, &before)).unwrap(),
        output
    );
    let removed = load(&root, json!(100));
    assert_ne!(removed.config_identity, before.config_identity);
    assert_eq!(
        serde_json::to_value(transform(&root, &removed)).unwrap(),
        output
    );
    let mut missing_watch = [
        root.join("tsconfig.json"),
        root.join("package.json"),
        root.join("node_modules/vue-router"),
        root.join("node_modules/vue-router/package.json"),
        entry.clone(),
    ]
    .map(|path| path.to_string_lossy().into_owned())
    .to_vec();
    missing_watch.sort();
    assert_eq!(removed.watched_files, missing_watch);
    std::fs::write(&entry, &original).unwrap();
    assert_eq!(
        load(&root, json!(100)).config_identity,
        before.config_identity
    );
    let reexport = entry.with_file_name("original.d.ts");
    std::fs::write(&reexport, &original).unwrap();
    std::fs::write(&entry, "export * from './original.js';\n").unwrap();
    let rebound = load(&root, json!(100));
    assert_ne!(rebound.config_identity, before.config_identity);
    assert_eq!(
        serde_json::to_value(transform(&root, &rebound)).unwrap(),
        output
    );
    let mut rebound_watch = expected_watch(&root, "index-BA_Jc-EQ");
    rebound_watch.extend(
        ["original.ts", "original.tsx", "original.d.ts"].map(|name| {
            entry
                .with_file_name(name)
                .to_string_lossy()
                .into_owned()
                .into()
        }),
    );
    rebound_watch.sort();
    assert_eq!(rebound.watched_files, rebound_watch);
    std::fs::remove_file(&reexport).unwrap();
    assert_eq!(
        serde_json::to_value(transform(&root, &load(&root, json!(100)))).unwrap(),
        output
    );
    install(&root, "published");
    assert_eq!(
        serde_json::to_value(transform(&root, &load(&root, json!(100)))).unwrap(),
        output
    );
    install(&root, "preview");
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{"vueCompilerOptions":{"plugins":[]}}"#,
    )
    .unwrap();
    let disabled = load(&root, json!(100));
    let unconfigured = generate_vue_content_mapper_transform_with_project_options(
        &root.join("src/pages/[id].vue"),
        PAGE,
        disabled.transform_options,
        None,
    )
    .unwrap();
    assert_eq!(
        serde_json::to_value(transform(&root, &disabled)).unwrap(),
        serde_json::to_value(unconfigured).unwrap()
    );
}

#[test]
fn resolver_redirections_never_publish_a_route_props_capability() {
    let (_guard, root) = project("preview");
    let legacy = generate_vue_content_mapper_transform_with_project_options(
        &root.join("src/pages/[id].vue"),
        PAGE,
        Default::default(),
        Some(&root),
    )
    .unwrap();
    for extension in [
        json!({"baseUrl":"."}),
        json!({"baseUrl":""}),
        json!({"baseUrl":null}),
        json!({"paths":null}),
        json!({"paths":[]}),
        json!({"paths":{"vue-router/experimental":["redirected.d.ts"]}}),
        json!({"paths":{"@/*":["src/*"]}}),
    ] {
        let mut compiler = json!({"rootDir":root,"moduleResolution":100});
        compiler
            .as_object_mut()
            .unwrap()
            .extend(extension.as_object().unwrap().clone());
        let config = ContentMapperProjectConfiguration::load_with_compiler_options(
            &root.join("tsconfig.json"),
            compiler.as_object().unwrap(),
        )
        .unwrap()
        .0;
        assert_eq!(
            config.watched_files,
            vec![root.join("tsconfig.json").to_string_lossy().into_owned()]
        );
        assert_eq!(
            serde_json::to_value(transform(&root, &config)).unwrap(),
            serde_json::to_value(&legacy).unwrap()
        );
    }
    let empty = ContentMapperProjectConfiguration::load_with_compiler_options(
        &root.join("tsconfig.json"),
        json!({"rootDir":root,"moduleResolution":100,"paths":{}})
            .as_object()
            .unwrap(),
    )
    .unwrap()
    .0;
    assert_eq!(
        serde_json::to_value(transform(&root, &empty)).unwrap(),
        serde_json::to_value(transform(&root, &load(&root, json!(100)))).unwrap()
    );
}
