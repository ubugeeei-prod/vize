use super::{discover_in_walk, write_file, write_native_preview_packages};
use std::fs;
use tempfile::TempDir;

#[test]
fn prefers_native_preview_binary_over_node_modules_bin_wrapper() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let wrapper = root.join("node_modules").join(".bin").join("tsgo");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"devDependencies":{"@typescript/native-preview":"7.0.0","provider":"1.0.0"}}"#,
    )
    .unwrap();
    let native = write_native_preview_packages(&root, "7.0.0");
    let provider = root.join("node_modules/provider");
    fs::create_dir_all(&provider).unwrap();
    fs::write(
        provider.join("package.json"),
        r#"{"name":"provider","bin":{"tsgo":"bin/tsgo.js"}}"#,
    )
    .unwrap();
    write_file(&wrapper);

    let resolved = discover_in_walk(std::slice::from_ref(&root));

    assert_eq!(resolved, Some(native.canonicalize().unwrap()));
}

#[test]
fn prefers_workspace_native_preview_over_nested_wrapper() {
    let temp_dir = TempDir::new().unwrap();
    let workspace_root = temp_dir.path().join("workspace");
    let nested = workspace_root.join("packages").join("demo");
    let wrapper = nested.join("node_modules").join(".bin").join("tsgo");
    fs::create_dir_all(&workspace_root).unwrap();
    fs::write(
        workspace_root.join("package.json"),
        r#"{"devDependencies":{"@typescript/native-preview":"7.0.0"}}"#,
    )
    .unwrap();
    let native = write_native_preview_packages(&workspace_root, "7.0.0");
    fs::create_dir_all(&nested).unwrap();
    fs::write(
        nested.join("package.json"),
        r#"{"devDependencies":{"provider":"1.0.0"}}"#,
    )
    .unwrap();
    let provider = nested.join("node_modules/provider");
    fs::create_dir_all(&provider).unwrap();
    fs::write(
        provider.join("package.json"),
        r#"{"name":"provider","bin":{"tsgo":"bin/tsgo.js"}}"#,
    )
    .unwrap();
    write_file(&wrapper);

    let resolved = discover_in_walk(&[nested]);

    assert_eq!(resolved, Some(native.canonicalize().unwrap()));
}

#[test]
fn falls_back_to_node_modules_bin_wrapper_when_no_native_binary_exists() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("project");
    let wrapper = root.join("node_modules").join(".bin").join("tsgo");
    fs::create_dir_all(&root).unwrap();
    fs::write(
        root.join("package.json"),
        r#"{"devDependencies":{"provider":"1.0.0"}}"#,
    )
    .unwrap();
    let provider = root.join("node_modules/provider");
    fs::create_dir_all(&provider).unwrap();
    fs::write(
        provider.join("package.json"),
        r#"{"name":"provider","bin":{"tsgo":"bin/tsgo.js"}}"#,
    )
    .unwrap();
    write_file(&wrapper);

    let resolved = discover_in_walk(&[root]);

    assert_eq!(resolved, Some(wrapper));
}
