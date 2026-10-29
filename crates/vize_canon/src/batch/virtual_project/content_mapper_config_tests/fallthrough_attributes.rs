use std::path::Path;

use super::load;
use vize_carton::config::VueVersion;

fn load_source(source: &str) -> super::ContentMapperProjectConfiguration {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tsconfig.json");
    std::fs::write(&path, source).unwrap();
    load(&path)
}

fn assert_flags(source: &str, enabled: bool, specified: bool) {
    let configuration = load_source(source);
    assert_eq!(
        configuration.transform_options.fallthrough_attributes,
        enabled
    );
    assert_eq!(
        configuration
            .transform_options
            .fallthrough_attributes_specified,
        specified
    );
    let checks = configuration.transform_options.virtual_ts_check_options();
    assert_eq!(checks.fallthrough_attributes, enabled);
    assert_eq!(checks.fallthrough_attributes_specified, specified);
}

#[test]
fn omitted_boolean_and_non_boolean_values_retain_exact_presence() {
    assert_flags("{}", false, false);
    assert_flags(
        r#"{"vueCompilerOptions":{"fallthroughAttributes":true}}"#,
        true,
        true,
    );
    assert_flags(
        r#"{"vueCompilerOptions":{"fallthroughAttributes":false}}"#,
        false,
        true,
    );
    assert_flags(
        r#"{"vueCompilerOptions":{"fallthroughAttributes":"off"}}"#,
        false,
        true,
    );
}

#[test]
fn inherited_child_false_overrides_true_and_remains_specified() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    std::fs::write(
        root.join("base.json"),
        r#"{"vueCompilerOptions":{"fallthroughAttributes":true}}"#,
    )
    .unwrap();
    let path = root.join("tsconfig.json");
    std::fs::write(
        &path,
        r#"{"extends":"./base.json","vueCompilerOptions":{"fallthroughAttributes":false}}"#,
    )
    .unwrap();
    let configuration = load(&path);
    assert!(!configuration.transform_options.fallthrough_attributes);
    assert!(
        configuration
            .transform_options
            .fallthrough_attributes_specified
    );
    let checks = configuration.transform_options.virtual_ts_check_options();
    assert!(!checks.fallthrough_attributes);
    assert!(checks.fallthrough_attributes_specified);
}

#[test]
fn omitted_and_explicit_false_have_distinct_configuration_identities() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tsconfig.json");
    std::fs::write(&path, "{}").unwrap();
    let omitted = load(&path);
    std::fs::write(
        &path,
        r#"{"vueCompilerOptions":{"fallthroughAttributes":false}}"#,
    )
    .unwrap();
    let explicit_false = load(&path);
    assert_eq!(omitted.watched_files, explicit_false.watched_files);
    assert_ne!(omitted.config_identity, explicit_false.config_identity);
}

#[test]
fn inferred_project_retains_omitted_presence() {
    let configuration = load(Path::new(""));
    assert!(!configuration.transform_options.fallthrough_attributes);
    assert!(
        !configuration
            .transform_options
            .fallthrough_attributes_specified
    );
}

#[test]
fn vue2_targets_reach_virtual_ts_with_omitted_presence() {
    for (source, expected) in [
        (r#"{"vueCompilerOptions":{"target":2}}"#, VueVersion::V2),
        (r#"{"vueCompilerOptions":{"target":2.7}}"#, VueVersion::V2_7),
    ] {
        let configuration = load_source(source);
        let checks = configuration.transform_options.virtual_ts_check_options();
        assert_eq!(checks.vue_compiler_target, Some(expected));
        assert!(!checks.fallthrough_attributes);
        assert!(!checks.fallthrough_attributes_specified);
    }
}
