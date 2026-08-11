use std::path::Path;

fn normalized(path: &Path) -> std::path::PathBuf {
    vize_carton::path::canonicalize_non_verbatim(path)
}

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
    let root = normalized(&root);
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
        normalized(&root.path().join("tsconfig.app.json"))
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
        normalized(&root.path().join("config/tsconfig.app.json"))
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
        normalized(&root.path().join("tsconfig.app.json"))
    );
}

#[test]
fn inherited_out_dir_is_default_excluded_from_referenced_ownership() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("dist")).unwrap();
    write(
        root.path(),
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./tsconfig.app.json"},{"path":"./tsconfig.generated.json"}]}"#,
    );
    write(
        root.path(),
        "tsconfig.base.json",
        r#"{"compilerOptions":{"outDir":"./dist"}}"#,
    );
    write(
        root.path(),
        "tsconfig.app.json",
        r#"{"extends":"./tsconfig.base.json"}"#,
    );
    write(
        root.path(),
        "tsconfig.generated.json",
        r#"{"include":["dist/**/*"]}"#,
    );
    let source = root.path().join("dist/App.vue");
    std::fs::write(&source, "<template />").unwrap();

    assert_eq!(
        super::effective_config_for_source(&root.path().join("tsconfig.json"), &source),
        normalized(&root.path().join("tsconfig.generated.json"))
    );
}

#[test]
fn ownership_follows_host_case_semantics_for_globs_and_paths() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("src")).unwrap();
    write(
        root.path(),
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./tsconfig.app.json"}]}"#,
    );
    write(
        root.path(),
        "tsconfig.app.json",
        r#"{"include":["src/**/*.vue"]}"#,
    );
    let canonical_root = normalized(root.path());
    let source_with_different_case = canonical_root.join("SRC/app.VUE");
    let root_config = canonical_root.join("tsconfig.json");
    let app_config = canonical_root.join("tsconfig.app.json");

    let mut insensitive = super::TsconfigOwnershipCache::with_options(
        super::TsconfigOwnershipOptions::with_case_sensitive(false),
    );
    assert_eq!(
        insensitive.effective_config_for_source(
            &root_config,
            &source_with_different_case,
            super::TsconfigSourceKind::Typed,
        ),
        app_config
    );

    let mut sensitive = super::TsconfigOwnershipCache::with_options(
        super::TsconfigOwnershipOptions::with_case_sensitive(true),
    );
    assert_eq!(
        sensitive.effective_config_for_source(
            &root_config,
            &source_with_different_case,
            super::TsconfigSourceKind::Typed,
        ),
        root_config
    );
}

#[test]
fn package_json_tsconfig_extends_contributes_inherited_allow_js() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("packages/app/src")).unwrap();
    std::fs::create_dir_all(root.path().join("node_modules/@scope/tsconfig/configs")).unwrap();
    write(
        root.path(),
        "tsconfig.json",
        r#"{"files":[],"references":[{"path":"./packages/app"}]}"#,
    );
    write(
        &root.path().join("packages/app"),
        "tsconfig.json",
        r#"{"extends":"@scope/tsconfig","include":["src/**/*"]}"#,
    );
    write(
        &root.path().join("node_modules/@scope/tsconfig"),
        "package.json",
        r#"{"tsconfig":"configs/vue.json"}"#,
    );
    write(
        &root.path().join("node_modules/@scope/tsconfig/configs"),
        "vue.json",
        r#"{"compilerOptions":{"allowJs":true}}"#,
    );
    let source = root.path().join("packages/app/src/main.js");
    std::fs::write(&source, "export const ok = true").unwrap();
    let mut cache = super::TsconfigOwnershipCache::default();

    assert_eq!(
        cache.effective_config_for_source(
            &root.path().join("tsconfig.json"),
            &source,
            super::TsconfigSourceKind::JavaScript,
        ),
        normalized(&root.path().join("packages/app/tsconfig.json"))
    );
}
