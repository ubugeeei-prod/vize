use super::{resolve_project_root, runtime_project_root};
use std::{fs, path::Path};

#[test]
fn runtime_selection_fixture_uses_selected_config_roots() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../../../tests/_fixtures/runtime/native-runtime-10694/selection.json"
    ))
    .unwrap();
    assert_eq!(
        fixture.get("schema").unwrap(),
        "vize.native-runtime-selection"
    );
    assert_eq!(fixture.get("version").unwrap(), 1);
    let mut ids = Vec::new();
    for case in fixture.get("routingCases").unwrap().as_array().unwrap() {
        ids.push(case.get("id").unwrap().as_str().unwrap());
        let root = tempfile::tempdir().unwrap();
        let package = root.path().join("package");
        fs::create_dir_all(package.join("src")).unwrap();
        fs::write(package.join("package.json"), "{}").unwrap();
        fs::create_dir_all(root.path().join("node_modules/vue")).unwrap();
        let expected = root
            .path()
            .join(case.get("expectedRoot").unwrap().as_str().unwrap());
        let selected = case
            .get("selectedConfig")
            .unwrap()
            .as_str()
            .map(|file| root.path().join(file));
        if let Some(config) = &selected {
            fs::create_dir_all(config.parent().unwrap()).unwrap();
            fs::write(config, r#"{"compilerOptions":{"strict":true}}"#).unwrap();
        }
        match case.get("kind").unwrap().as_str().unwrap() {
            "config-parent" => assert_eq!(
                runtime_project_root(&package, selected.as_deref()),
                expected
            ),
            "nearest-package" => assert_eq!(
                resolve_project_root(&package.join("src/App.vue").to_string_lossy()),
                expected
            ),
            _ => panic!("unknown routing case"),
        }
    }
    assert_eq!(
        ids,
        [
            "selected-config-root",
            "closest-package-before-ancestor-vue",
            "no-config-package-root"
        ]
    );
}

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
