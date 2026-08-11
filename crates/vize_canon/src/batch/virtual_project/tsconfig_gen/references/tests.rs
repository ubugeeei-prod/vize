use std::path::Path;

fn write(root: &Path, name: &str, content: &str) {
    std::fs::write(root.join(name), content).unwrap();
}

#[test]
fn file_and_directory_references_resolve_missing_ones_drop() {
    let root = std::env::temp_dir().join(format!("vize-refs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("sub")).unwrap();
    write(
        &root,
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./tsconfig.app.json"},{"path":"./sub"},{"path":"./missing.json"}]}"#,
    );
    write(&root, "tsconfig.app.json", "{}");
    write(&root.join("sub"), "tsconfig.json", "{}");

    let configs = super::referenced_project_configs(&root.join("tsconfig.json"));
    let names = configs
        .iter()
        .map(|config| {
            config
                .strip_prefix(&root)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<Vec<_>>();
    assert_eq!(names, ["tsconfig.app.json", "sub/tsconfig.json"]);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_config_without_references_yields_nothing() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "tsconfig.json", r#"{"compilerOptions":{}}"#);
    assert!(super::referenced_project_configs(&root.path().join("tsconfig.json")).is_empty());
}

#[test]
fn selects_the_only_referenced_project_that_includes_the_source() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./tsconfig.app.json"},{"path":"./tsconfig.node.json"}]}"#,
    );
    write(
        root.path(),
        "tsconfig.app.json",
        r#"{"include":["src/**/*"]}"#,
    );
    write(
        root.path(),
        "tsconfig.node.json",
        r#"{"include":["vite.config.ts"]}"#,
    );
    std::fs::create_dir_all(root.path().join("src")).unwrap();
    let source = root.path().join("src/App.vue");
    std::fs::write(&source, "<template />").unwrap();
    assert_eq!(
        super::effective_config_for_source(&root.path().join("tsconfig.json"), &source),
        root.path().join("tsconfig.app.json")
    );
}

#[test]
fn inherited_include_keeps_its_declaring_config_anchor() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("config/src")).unwrap();
    write(
        root.path(),
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./config/tsconfig.app.json"}]}"#,
    );
    write(
        &root.path().join("config"),
        "tsconfig.base.json",
        r#"{"include":["src/**/*"]}"#,
    );
    write(
        &root.path().join("config"),
        "tsconfig.app.json",
        r#"{"extends":"./tsconfig.base.json","compilerOptions":{"strict":true}}"#,
    );
    let source = root.path().join("config/src/App.vue");
    std::fs::write(&source, "<template />").unwrap();
    assert_eq!(
        super::effective_config_for_source(&root.path().join("tsconfig.json"), &source),
        root.path().join("config/tsconfig.app.json")
    );
}

#[test]
fn nested_solution_references_reach_the_leaf_config() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("src")).unwrap();
    write(
        root.path(),
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./tsconfig.workspace.json"}]}"#,
    );
    write(
        root.path(),
        "tsconfig.workspace.json",
        r#"{"files":[],"references":[{"path":"./tsconfig.app.json"}]}"#,
    );
    write(
        root.path(),
        "tsconfig.app.json",
        r#"{"include":["src/**/*"]}"#,
    );
    let source = root.path().join("src/App.vue");
    std::fs::write(&source, "<template />").unwrap();
    assert_eq!(
        super::effective_config_for_source(&root.path().join("tsconfig.json"), &source),
        root.path().join("tsconfig.app.json")
    );
}
