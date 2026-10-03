use std::path::{Path, PathBuf};

use super::EditorProjectConfiguration;

#[test]
fn excluded_component_augmentation_keeps_the_actual_diagnostic_project_boundary() {
    let workspace = tempfile::tempdir().unwrap();
    let host = write(&workspace.path().join("src/App.vue"), "<template />");
    write(
        &workspace.path().join("tsconfig.json"),
        r#"{"include":["src/**/*"]}"#,
    );
    let components = write(
        &workspace.path().join(".nuxt/components.d.ts"),
        "export {};",
    );
    let configuration = EditorProjectConfiguration::for_source(&host, Some(workspace.path()), None);
    assert!(
        configuration
            .reference_paths(&[components.clone()])
            .is_empty()
    );
    assert_eq!(
        configuration.component_reference_paths(&[components.clone(), components.clone()]),
        vec![components]
    );
}

#[test]
fn component_compatibility_does_not_cross_a_nested_or_foreign_project() {
    let workspace = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let host = write(&workspace.path().join("src/App.vue"), "<template />");
    write(
        &workspace.path().join("tsconfig.json"),
        r#"{"include":["src/**/*"]}"#,
    );
    write(
        &workspace.path().join("nested/tsconfig.json"),
        r#"{"include":["src/**/*"]}"#,
    );
    let nested = write(
        &workspace.path().join("nested/generated/components.d.ts"),
        "export {};",
    );
    let foreign = write(&outside.path().join("components.d.ts"), "export {};");
    let missing = workspace.path().join("missing.d.ts");
    let non_declaration = write(&workspace.path().join("components.ts"), "export {};");
    let configuration = EditorProjectConfiguration::for_source(&host, Some(workspace.path()), None);
    assert!(
        configuration
            .component_reference_paths(&[nested, foreign, missing, non_declaration])
            .is_empty()
    );
}

#[test]
fn component_compatibility_requires_a_readable_unique_source_owner() {
    let workspace = tempfile::tempdir().unwrap();
    let host = write(&workspace.path().join("src/App.vue"), "<template />");
    let component = write(
        &workspace.path().join("generated/components.d.ts"),
        "export {};",
    );
    let config = workspace.path().join("tsconfig.json");
    for contents in [r#"{"files":[]}"#, "not json"] {
        write(&config, contents);
        let configuration =
            EditorProjectConfiguration::for_source(&host, Some(workspace.path()), None);
        assert!(
            configuration
                .component_reference_paths(&[component.clone()])
                .is_empty()
        );
    }
    write(
        &config,
        r#"{"include":["src/**/*"],"references":[{"path":"duplicate.json"}]}"#,
    );
    write(
        &workspace.path().join("duplicate.json"),
        r#"{"include":["src/**/*"]}"#,
    );
    let configuration = EditorProjectConfiguration::for_source(&host, Some(workspace.path()), None);
    assert!(
        configuration
            .component_reference_paths(&[component])
            .is_empty()
    );
}

#[test]
fn referenced_source_owner_limits_component_compatibility_to_its_physical_root() {
    let workspace = tempfile::tempdir().unwrap();
    let shell = write(
        &workspace.path().join("tsconfig.json"),
        r#"{"files":[],"references":[{"path":"app"},{"path":"sibling"}]}"#,
    );
    let host = write(&workspace.path().join("app/src/App.vue"), "<template />");
    for root in ["app", "sibling"] {
        write(
            &workspace.path().join(root).join("tsconfig.json"),
            r#"{"include":["src/**/*"]}"#,
        );
    }
    let own = write(
        &workspace.path().join("app/generated/components.d.ts"),
        "export {};",
    );
    let sibling = write(
        &workspace.path().join("sibling/generated/components.d.ts"),
        "export {};",
    );
    let configuration =
        EditorProjectConfiguration::for_source(&host, Some(workspace.path()), Some(&shell));
    assert_eq!(
        configuration.component_reference_paths(&[sibling, own.clone()]),
        vec![own]
    );
}

fn write(path: &Path, source: &str) -> PathBuf {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, source).unwrap();
    vize_carton::path::canonicalize_non_verbatim(path)
}
