use std::path::{Path, PathBuf};

use super::materialize_runtime_dependencies;
use super::resolver::{
    VueRuntimePackages, resolve_package, resolve_vue_package, resolve_vue_runtime_packages,
    with_test_env_overrides,
};
use super::stubs::VUE_RUNTIME_CORE_STUB_TYPES;

#[test]
fn explicit_runtime_packages_override_project_packages() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    let explicit = temp.path().join("explicit");
    create_package(&project, "node_modules/vue");
    create_package(&project, "node_modules/@vue/runtime-dom");
    create_package(&project, "node_modules/vite");
    let explicit_vue = create_package(&explicit, "vue");
    let explicit_runtime_dom = create_package(&explicit, "@vue/runtime-dom");
    let explicit_vite = create_package(&explicit, "vite");

    with_test_env_overrides(
        &[
            ("VIZE_VUE_PACKAGE", Some(explicit_vue.as_path())),
            ("VIZE_VUE_NAMESPACE_PACKAGE", None),
            (
                "VIZE_VUE_RUNTIME_DOM_PACKAGE",
                Some(explicit_runtime_dom.as_path()),
            ),
            ("VIZE_VITE_PACKAGE", Some(explicit_vite.as_path())),
            ("VIZE_RUNTIME_NODE_MODULES", None),
            (
                "VIZE_TEST_WORKSPACE_NODE_MODULES",
                Some(Path::new("__none__")),
            ),
        ],
        || {
            assert_eq!(resolve_vue_package(&project), Some(explicit_vue.clone()));
            assert_eq!(
                resolve_vue_runtime_packages(&project, &explicit_vue),
                VueRuntimePackages::RuntimeDom(explicit_runtime_dom.clone())
            );
            assert_eq!(
                resolve_package(&project, "vite"),
                Some(explicit_vite.clone())
            );
        },
    );
}

#[test]
fn explicit_vue_namespace_overrides_explicit_runtime_dom() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    let vue = create_package(&project, "node_modules/vue");
    let namespace = create_package(&temp.path().join("explicit"), "@vue");
    create_package(&namespace, "runtime-dom");
    let runtime_dom = create_package(&temp.path().join("explicit"), "@vue-runtime-dom");

    with_test_env_overrides(
        &[
            ("VIZE_VUE_PACKAGE", None),
            ("VIZE_VUE_NAMESPACE_PACKAGE", Some(namespace.as_path())),
            ("VIZE_VUE_RUNTIME_DOM_PACKAGE", Some(runtime_dom.as_path())),
            ("VIZE_VITE_PACKAGE", None),
            ("VIZE_RUNTIME_NODE_MODULES", None),
            (
                "VIZE_TEST_WORKSPACE_NODE_MODULES",
                Some(Path::new("__none__")),
            ),
        ],
        || {
            assert_eq!(
                resolve_vue_runtime_packages(&project, &vue),
                VueRuntimePackages::Namespace(namespace.clone())
            );
        },
    );
}

#[test]
fn runtime_node_modules_supply_vue_and_vite_fallbacks() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    let runtime_node_modules = temp.path().join("runtime_node_modules");
    let runtime_vue = create_package(&runtime_node_modules, "vue");
    let runtime_dom = create_package(&runtime_node_modules, "@vue/runtime-dom");
    let runtime_vite = create_package(&runtime_node_modules, "vite");

    with_test_env_overrides(
        &[
            ("VIZE_VUE_PACKAGE", None),
            ("VIZE_VUE_NAMESPACE_PACKAGE", None),
            ("VIZE_VUE_RUNTIME_DOM_PACKAGE", None),
            ("VIZE_VITE_PACKAGE", None),
            (
                "VIZE_RUNTIME_NODE_MODULES",
                Some(runtime_node_modules.as_path()),
            ),
            (
                "VIZE_TEST_WORKSPACE_NODE_MODULES",
                Some(Path::new("__none__")),
            ),
        ],
        || {
            assert_eq!(resolve_vue_package(&project), Some(runtime_vue.clone()));
            assert_eq!(
                resolve_package(&project, "@vue/runtime-dom"),
                Some(runtime_dom.clone())
            );
            assert_eq!(
                resolve_package(&project, "vite"),
                Some(runtime_vite.clone())
            );
        },
    );
}

#[test]
fn materialized_runtime_dom_also_links_adjacent_runtime_core() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    let explicit = temp.path().join("explicit");
    let virtual_root = temp.path().join("virtual");
    let runtime_dom = create_package_with_types(
        &explicit,
        "@vue/runtime-dom",
        "export const runtimeDomLinked: unique symbol;",
    );
    create_package_with_types(
        &explicit,
        "@vue/runtime-core",
        "export const runtimeCoreLinked: unique symbol;",
    );

    with_test_env_overrides(
        &[
            ("VIZE_VUE_PACKAGE", None),
            ("VIZE_VUE_NAMESPACE_PACKAGE", None),
            ("VIZE_VUE_RUNTIME_DOM_PACKAGE", Some(runtime_dom.as_path())),
            ("VIZE_VITE_PACKAGE", None),
            ("VIZE_RUNTIME_NODE_MODULES", None),
            (
                "VIZE_TEST_WORKSPACE_NODE_MODULES",
                Some(Path::new("__none__")),
            ),
        ],
        || {
            materialize_runtime_dependencies(&project, &virtual_root, &[]).unwrap();
            assert_eq!(
                read_virtual_runtime_types(&virtual_root, "runtime-dom"),
                "export const runtimeDomLinked: unique symbol;"
            );
            assert_eq!(
                read_virtual_runtime_types(&virtual_root, "runtime-core"),
                "export const runtimeCoreLinked: unique symbol;"
            );
        },
    );
}

#[test]
fn materialized_runtime_dom_writes_runtime_core_stub_when_core_is_absent() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    let explicit = temp.path().join("explicit");
    let virtual_root = temp.path().join("virtual");
    let runtime_dom = create_package_with_types(
        &explicit,
        "@vue/runtime-dom",
        "export const runtimeDomLinked: unique symbol;",
    );

    with_test_env_overrides(
        &[
            ("VIZE_VUE_PACKAGE", None),
            ("VIZE_VUE_NAMESPACE_PACKAGE", None),
            ("VIZE_VUE_RUNTIME_DOM_PACKAGE", Some(runtime_dom.as_path())),
            ("VIZE_VITE_PACKAGE", None),
            ("VIZE_RUNTIME_NODE_MODULES", None),
            (
                "VIZE_TEST_WORKSPACE_NODE_MODULES",
                Some(Path::new("__none__")),
            ),
        ],
        || {
            materialize_runtime_dependencies(&project, &virtual_root, &[]).unwrap();
            assert_eq!(
                read_virtual_runtime_types(&virtual_root, "runtime-dom"),
                "export const runtimeDomLinked: unique symbol;"
            );
            assert_eq!(
                read_virtual_runtime_types(&virtual_root, "runtime-core"),
                VUE_RUNTIME_CORE_STUB_TYPES
            );
        },
    );
}

fn create_package(root: &Path, relative: &str) -> PathBuf {
    let dir = root.join(relative);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("package.json"), "{}").unwrap();
    dir
}

fn create_package_with_types(root: &Path, relative: &str, types: &str) -> PathBuf {
    let dir = create_package(root, relative);
    std::fs::write(dir.join("index.d.ts"), types).unwrap();
    dir
}

fn read_virtual_runtime_types(virtual_root: &Path, package: &str) -> String {
    std::fs::read_to_string(
        virtual_root
            .join("node_modules")
            .join("@vue")
            .join(package)
            .join("index.d.ts"),
    )
    .unwrap()
}
