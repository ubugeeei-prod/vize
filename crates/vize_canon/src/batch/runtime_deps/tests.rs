use std::path::{Path, PathBuf};

use super::resolver::{
    VueRuntimePackages, resolve_package, resolve_vue_package, resolve_vue_runtime_packages,
};

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

    with_env_overrides(
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

    with_env_overrides(
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

    with_env_overrides(
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

fn create_package(root: &Path, relative: &str) -> PathBuf {
    let dir = root.join(relative);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("package.json"), "{}").unwrap();
    dir
}

fn with_env_overrides<T>(vars: &[(&str, Option<&Path>)], run: impl FnOnce() -> T) -> T {
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct EnvGuard {
        previous: Vec<(String, Option<std::ffi::OsString>)>,
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (name, value) in self.previous.drain(..).rev() {
                match value {
                    Some(value) => unsafe { std::env::set_var(name, value) },
                    None => unsafe { std::env::remove_var(name) },
                }
            }
        }
    }

    let _lock = ENV_LOCK.lock().unwrap();
    let previous = vars
        .iter()
        .map(|(name, _)| ((*name).to_owned(), std::env::var_os(name)))
        .collect();
    for (name, value) in vars {
        match value {
            Some(value) => unsafe { std::env::set_var(name, value) },
            None => unsafe { std::env::remove_var(name) },
        }
    }
    let _guard = EnvGuard { previous };
    run()
}
