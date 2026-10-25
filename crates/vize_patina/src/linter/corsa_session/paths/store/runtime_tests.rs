use super::{resolve_project_root, runtime_project_root};
use std::{fs, path::Path};

#[test]
fn nearest_authored_package_precedes_an_ancestor_vue_install() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path();
    let nested = workspace.join("packages/app");
    fs::create_dir_all(nested.join("src")).unwrap();
    fs::create_dir_all(workspace.join("node_modules/vue")).unwrap();
    fs::write(nested.join("package.json"), "{}").unwrap();
    assert_eq!(
        resolve_project_root(&nested.join("src/App.vue").to_string_lossy()),
        nested
    );
}

#[test]
fn selected_config_parent_is_the_runtime_lookup_root() {
    let authored = Path::new("/workspace/package");
    let config = Path::new("/workspace/selected/tsconfig.app.json");
    assert_eq!(
        runtime_project_root(authored, Some(config)),
        Path::new("/workspace/selected")
    );
    assert_eq!(runtime_project_root(authored, None), authored);
    assert_eq!(
        runtime_project_root(authored, Some(Path::new("tsconfig.json"))),
        authored
    );
}

#[test]
fn vue_install_remains_a_fallback_when_no_package_manifest_exists() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join("node_modules/vue")).unwrap();
    fs::create_dir_all(root.join("src/nested")).unwrap();
    assert_eq!(
        resolve_project_root(&root.join("src/nested/App.vue").to_string_lossy()),
        root
    );
}
