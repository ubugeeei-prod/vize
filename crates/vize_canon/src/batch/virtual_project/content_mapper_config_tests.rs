//! Project configuration loading for the official TypeScript content mapper.

use super::generate_vue_content_mapper_transform_with_options;
use super::{ContentMapperProjectConfiguration, ContentMapperTransformOptions};
use crate::batch::error::CorsaError;
use std::io::ErrorKind;
use std::path::Path;
use vize_l0::config::VueVersion;

fn load(path: &Path) -> ContentMapperProjectConfiguration {
    ContentMapperProjectConfiguration::load(path).unwrap()
}

fn expected_options(
    mut options: ContentMapperTransformOptions,
    components: bool,
    unknown: bool,
    fallthrough: bool,
    directives: bool,
    css: bool,
) -> ContentMapperTransformOptions {
    options.check_unknown_components = components;
    options.check_unknown_props = unknown;
    options.check_unknown_fallthrough_props = fallthrough;
    options.check_unknown_directives = directives;
    options.strict_css_modules = css;
    options
}

fn expected_watches(root: &Path, names: &[&str]) -> Vec<vize_l0::String> {
    let mut paths: Vec<_> = names
        .iter()
        .map(|name| root.join(name).to_string_lossy().into_owned().into())
        .collect();
    paths.sort();
    paths.dedup();
    paths
}

fn expected_canonical_watch(path: &Path) -> vize_l0::String {
    let text = path.canonicalize().unwrap().to_string_lossy().into_owned();
    // The host returns ordinary DOS/UNC paths, not Windows verbatim paths.
    // Form the oracle from the OS result without calling the resolver's helper.
    #[cfg(windows)]
    let text = text
        .strip_prefix(r"\\?\UNC\")
        .map(|unc| format!(r"\\{unc}"))
        .or_else(|| text.strip_prefix(r"\\?\").map(str::to_owned))
        .unwrap_or(text);
    text.into()
}

#[test]
fn inferred_identity_retains_the_complete_sha256_digest() {
    let configuration = load(Path::new(""));
    assert_eq!(
        configuration.config_identity.as_str(),
        "vize-content-mapper-v1:7a38905296006c575d7d45d1d881690a3ee87361ed67ce64642e2504482da818"
    );
}

#[test]
fn configured_defaults_and_inferred_defaults_remain_distinct() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tsconfig.json");
    std::fs::write(&path, "{}").unwrap();
    let configured = load(&path);
    assert_eq!(
        configured.transform_options,
        expected_options(Default::default(), false, false, false, false, false)
    );
    assert_eq!(
        configured.watched_files,
        expected_watches(directory.path(), &["tsconfig.json"])
    );
    let inferred = load(Path::new(""));
    assert_eq!(
        inferred.transform_options,
        expected_options(Default::default(), false, true, false, false, false)
    );
    assert_eq!(
        inferred.watched_files,
        expected_watches(directory.path(), &[])
    );
    assert_ne!(configured.config_identity, inferred.config_identity);
}

#[test]
fn shared_chain_obeys_jsonc_array_precedence_and_explicit_false() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    std::fs::write(
        root.join("left.json"),
        r#"{
        // A real inherited option, including a trailing comma.
        "vueCompilerOptions": {"checkUnknownProps": true, "strictCssModules": true,},
    }"#,
    )
    .unwrap();
    std::fs::write(
        root.join("right.json"),
        r#"{
        "extends": "./left.json", "vueCompilerOptions": {"strictCssModules": false}
    }"#,
    )
    .unwrap();
    let path = root.join("tsconfig.json");
    std::fs::write(
        &path,
        r#"{
        "extends": ["./left.json", "./right.json"],
        "vueCompilerOptions": {"strictTemplates": true, "checkUnknownProps": false}
    }"#,
    )
    .unwrap();
    let configuration = load(&path);
    assert_eq!(
        configuration.transform_options,
        expected_options(Default::default(), true, false, false, true, false)
    );
    assert_eq!(
        configuration.watched_files,
        expected_watches(root, &["tsconfig.json", "left.json", "right.json"])
    );
}

#[test]
fn package_tsconfig_field_and_real_parent_are_in_the_dynamic_closure() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let package = root.join("node_modules/tsconfig-preset");
    std::fs::create_dir_all(&package).unwrap();
    std::fs::write(package.join("package.json"), r#"{"tsconfig":"vue.json"}"#).unwrap();
    std::fs::write(
        package.join("vue.json"),
        r#"{
        "vueCompilerOptions": {"checkUnknownProps":true,"strictCssModules":true}
    }"#,
    )
    .unwrap();
    let path = root.join("tsconfig.json");
    std::fs::write(&path, r#"{"extends":"tsconfig-preset"}"#).unwrap();
    let configuration = load(&path);
    assert_eq!(
        configuration.transform_options,
        expected_options(Default::default(), false, true, true, false, true)
    );
    let mut expected = expected_watches(
        root,
        &[
            "tsconfig.json",
            "node_modules/tsconfig-preset/package.json",
            "node_modules/tsconfig-preset/vue.json",
        ],
    );
    // Resolution observes the lexical probe, then reads its canonical target.
    // Both are retained when the temporary directory itself has a symlink alias.
    expected.push(expected_canonical_watch(&package.join("vue.json")));
    expected.sort();
    expected.dedup();
    assert_eq!(configuration.watched_files, expected);
}

#[test]
fn dynamic_identity_is_stable_and_tracks_inherited_option_changes() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let path = root.join("tsconfig.json");
    let base = root.join("base.json");
    std::fs::write(&path, r#"{"extends":"./base.json"}"#).unwrap();
    std::fs::write(&base, r#"{"vueCompilerOptions":{"strictTemplates":true}}"#).unwrap();
    let before = load(&path);
    assert_eq!(
        before.transform_options,
        expected_options(Default::default(), true, true, false, true, false)
    );
    assert_eq!(before.config_identity, load(&path).config_identity);
    std::fs::write(&base, r#"{"vueCompilerOptions":{"strictTemplates":true,"checkUnknownProps":false,"strictCssModules":true}}"#).unwrap();
    let after = load(&path);
    assert_eq!(
        after.transform_options,
        expected_options(Default::default(), true, false, false, true, true)
    );
    assert_ne!(before.config_identity, after.config_identity);
    assert_eq!(before.watched_files, after.watched_files);
    assert_eq!(
        after.watched_files,
        expected_watches(root, &["tsconfig.json", "base.json"])
    );
}

#[test]
fn cycle_cut_keeps_the_existing_chain_precedence() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let path = root.join("tsconfig.json");
    std::fs::write(
        &path,
        r#"{"extends":"./base.json","vueCompilerOptions":{"checkUnknownProps":false}}"#,
    )
    .unwrap();
    std::fs::write(root.join("base.json"), r#"{"extends":"./tsconfig.json","vueCompilerOptions":{"checkUnknownProps":true,"strictCssModules":true}}"#).unwrap();
    let configuration = load(&path);
    assert_eq!(
        configuration.transform_options,
        expected_options(Default::default(), false, false, false, false, true)
    );
    assert_eq!(
        configuration.watched_files,
        expected_watches(root, &["tsconfig.json", "base.json"])
    );
}

#[test]
fn malformed_inherited_configuration_is_not_replaced_by_defaults() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tsconfig.json");
    std::fs::write(&path, r#"{"extends":"./base.json"}"#).unwrap();
    std::fs::write(directory.path().join("base.json"), "{ invalid JSON").unwrap();
    let CorsaError::Io(error) = ContentMapperProjectConfiguration::load(&path).unwrap_err() else {
        panic!("malformed inherited configuration must produce an IO error");
    };
    assert_eq!(error.kind(), ErrorKind::InvalidData);
    assert_eq!(
        error.to_string().as_str(),
        vize_l0::cstr!(
            "Invalid tsconfig '{}': JSON parse error: key must be a string at line 1 column 3",
            directory.path().join("base.json").display()
        )
        .as_str()
    );
    std::fs::remove_file(directory.path().join("base.json")).unwrap();
    let CorsaError::Io(error) = ContentMapperProjectConfiguration::load(&path).unwrap_err() else {
        panic!("missing inherited configuration must produce an IO error");
    };
    assert_eq!(error.kind(), ErrorKind::NotFound);
    assert_eq!(
        error.to_string().as_str(),
        vize_l0::cstr!(
            "Cannot resolve extended tsconfig './base.json' from '{}'",
            path.display()
        )
        .as_str()
    );
}

fn has_fragment(code: &str, needle: &str) -> bool {
    code.split(needle).nth(1).is_some()
}

const INTRINSIC_SFC: &str =
    "<script setup lang=\"ts\"></script>\n<template><div x-small></div></template>\n";

#[test]
fn vue3_target_drops_intrinsic_attribute_excess_checks() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tsconfig.json");
    std::fs::write(
        &path,
        r#"{"vueCompilerOptions":{"target":3,"checkUnknownProps":true}}"#,
    )
    .unwrap();
    let configured = load(&path);
    assert_eq!(
        configured.transform_options.vue_compiler_target,
        Some(VueVersion::V3)
    );
    let generated = generate_vue_content_mapper_transform_with_options(
        Path::new("src/Native.vue"),
        INTRINSIC_SFC,
        configured.transform_options,
    )
    .unwrap();
    let excess = has_fragment(&generated.text, "__vize_native_key_");
    assert!(!excess, "{}", generated.text);

    std::fs::write(
        &path,
        r#"{"vueCompilerOptions":{"checkUnknownProps":true}}"#,
    )
    .unwrap();
    let omitted = load(&path);
    assert_eq!(omitted.transform_options.vue_compiler_target, None);
    let checked = generate_vue_content_mapper_transform_with_options(
        Path::new("src/Native.vue"),
        INTRINSIC_SFC,
        omitted.transform_options,
    )
    .unwrap();
    let still_checked = has_fragment(&checked.text, "__vize_native_key_");
    assert!(still_checked, "{}", checked.text);
}

#[test]
fn unsupported_vue_compiler_target_is_configuration_error() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tsconfig.json");
    std::fs::write(&path, r#"{"vueCompilerOptions":{"target":1}}"#).unwrap();
    let CorsaError::Configuration(message) =
        ContentMapperProjectConfiguration::load(&path).unwrap_err()
    else {
        panic!("unsupported target must be a configuration error");
    };
    let names_target = has_fragment(message.as_str(), "vueCompilerOptions.target");
    assert!(names_target, "{message}");
}

fn is_vue_manifest(path: &str) -> bool {
    let mut parts = path.rsplit(['/', '\\']);
    let file = parts.next();
    let package = parts.next();
    let modules = parts.next();
    file == Some("package.json") && package == Some("vue") && modules == Some("node_modules")
}

#[test]
fn auto_target_retains_nearer_missing_candidates_and_repair_without_numeric_probes() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let project = root.join("日本語").join("child");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(root.join("package.json"), r#"{"name":"fixture"}"#).unwrap();
    let outer = root.join("node_modules/vue/package.json");
    std::fs::create_dir_all(outer.parent().unwrap()).unwrap();
    std::fs::write(&outer, r#"{"version":"2.7.16"}"#).unwrap();
    let config = project.join("tsconfig.json");
    std::fs::write(&config, r#"{"vueCompilerOptions":{"target":"auto"}}"#).unwrap();
    let before = load(&config);
    assert_eq!(
        before.transform_options.vue_compiler_target,
        Some(VueVersion::V2_7)
    );
    let mut expected = expected_watches(
        root,
        &[
            "日本語/child/tsconfig.json",
            "日本語/child/package.json",
            "日本語/package.json",
            "package.json",
            "日本語/child/node_modules/vue",
            "日本語/child/node_modules/vue/package.json",
            "日本語/node_modules/vue",
            "日本語/node_modules/vue/package.json",
            "node_modules/vue",
            "node_modules/vue/package.json",
        ],
    );
    expected.sort();
    assert_eq!(before.watched_files, expected);
    let nearer = project.join("node_modules/vue/package.json");
    std::fs::create_dir_all(nearer.parent().unwrap()).unwrap();
    std::fs::write(&nearer, r#"{"version":"3.5.0"}"#).unwrap();
    let created = load(&config);
    assert_eq!(
        created.transform_options.vue_compiler_target,
        Some(VueVersion::V3)
    );
    assert_ne!(created.config_identity, before.config_identity);
    assert!(
        !created
            .watched_files
            .iter()
            .any(|path| path.as_str() == outer.to_string_lossy())
    );
    std::fs::remove_file(&nearer).unwrap();
    let removed = load(&config);
    assert_eq!(removed.config_identity, before.config_identity);
    assert_eq!(removed.watched_files, before.watched_files);
    std::fs::write(&nearer, r#"{"version":"invalid"}"#).unwrap();
    assert!(ContentMapperProjectConfiguration::load(&config).is_err());
    std::fs::write(&nearer, r#"{"version":"3.5.0"}"#).unwrap();
    let repaired = load(&config);
    assert_eq!(repaired.config_identity, created.config_identity);
    assert_eq!(repaired.watched_files, created.watched_files);
    for source in [r#"{"vueCompilerOptions":{"target":3}}"#, "{}"] {
        std::fs::write(&config, source).unwrap();
        assert_eq!(
            load(&config).watched_files,
            expected_watches(root, &["日本語/child/tsconfig.json"])
        );
    }
}

#[test]
fn auto_target_watches_installed_vue_and_identity_follows_the_resolved_dialect() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let manifest = root.join("node_modules/vue/package.json");
    std::fs::create_dir_all(manifest.parent().unwrap()).unwrap();
    std::fs::write(&manifest, r#"{"version":"2.7.16"}"#).unwrap();
    let path = root.join("tsconfig.json");
    std::fs::write(
        &path,
        r#"{"vueCompilerOptions":{"target":"auto","checkUnknownProps":true}}"#,
    )
    .unwrap();
    let vue2 = load(&path);
    assert_eq!(
        vue2.transform_options.vue_compiler_target,
        Some(VueVersion::V2_7)
    );
    let watched = vue2
        .watched_files
        .iter()
        .any(|file| is_vue_manifest(file.as_str()));
    assert!(watched);
    assert_eq!(vue2.config_identity, load(&path).config_identity);

    std::fs::write(&manifest, r#"{"version":"3.5.0"}"#).unwrap();
    let vue3 = load(&path);
    assert_eq!(
        vue3.transform_options.vue_compiler_target,
        Some(VueVersion::V3)
    );
    assert_ne!(vue2.config_identity, vue3.config_identity);

    std::fs::write(&path, r#"{"vueCompilerOptions":{"target":3}}"#).unwrap();
    let numeric = load(&path);
    let numeric_watches_vue = numeric
        .watched_files
        .iter()
        .any(|file| is_vue_manifest(file.as_str()));
    assert!(!numeric_watches_vue);
}
