use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};

#[cfg(test)]
use std::cell::RefCell;

const VIZE_VITE_PACKAGE_ENV: &str = "VIZE_VITE_PACKAGE";
const VIZE_VUE_PACKAGE_ENV: &str = "VIZE_VUE_PACKAGE";
const VIZE_VUE_NAMESPACE_PACKAGE_ENV: &str = "VIZE_VUE_NAMESPACE_PACKAGE";
const VIZE_VUE_RUNTIME_DOM_PACKAGE_ENV: &str = "VIZE_VUE_RUNTIME_DOM_PACKAGE";
const VIZE_RUNTIME_NODE_MODULES_ENV: &str = "VIZE_RUNTIME_NODE_MODULES";
#[cfg(test)]
const VIZE_TEST_WORKSPACE_NODE_MODULES_ENV: &str = "VIZE_TEST_WORKSPACE_NODE_MODULES";

#[cfg(test)]
thread_local! {
    static TEST_ENV_OVERRIDE_STACK: RefCell<Vec<Vec<(&'static str, Option<OsString>)>>> =
        const { RefCell::new(Vec::new()) };
}

#[cfg(test)]
struct TestEnvOverrideGuard;

#[cfg(test)]
impl Drop for TestEnvOverrideGuard {
    fn drop(&mut self) {
        TEST_ENV_OVERRIDE_STACK.with(|stack| {
            debug_assert!(stack.borrow_mut().pop().is_some());
        });
    }
}

/// Scope test-only resolver overrides to the calling test thread.
///
/// Rust unit tests share a process and run concurrently. Mutating the real
/// environment here would let unrelated resolver calls observe partial fixture
/// state, so overrides live in a thread-local stack instead.
#[cfg(test)]
pub(crate) fn with_test_env_overrides<T>(
    vars: &[(&'static str, Option<&Path>)],
    run: impl FnOnce() -> T,
) -> T {
    let frame = vars
        .iter()
        .map(|(name, value)| (*name, value.map(Path::as_os_str).map(OsString::from)))
        .collect();
    TEST_ENV_OVERRIDE_STACK.with(|stack| stack.borrow_mut().push(frame));
    let _guard = TestEnvOverrideGuard;
    run()
}

#[cfg(test)]
pub(crate) fn test_env_var_os(name: &str) -> Option<OsString> {
    env_var_os(name)
}

fn env_var_os(name: &str) -> Option<OsString> {
    #[cfg(test)]
    {
        let overridden = TEST_ENV_OVERRIDE_STACK.with(|stack| {
            for frame in stack.borrow().iter().rev() {
                if let Some((_, value)) =
                    frame.iter().rev().find(|(candidate, _)| *candidate == name)
                {
                    return Some(value.clone());
                }
            }
            None
        });
        if let Some(value) = overridden {
            return value;
        }
    }

    env::var_os(name)
}

#[derive(Debug, Eq, PartialEq)]
pub(super) enum VueRuntimePackages {
    Namespace(PathBuf),
    RuntimeDom(PathBuf),
    Stub,
}

pub(super) fn resolve_vue_runtime_packages(
    project_root: &Path,
    vue_source: &Path,
) -> VueRuntimePackages {
    if let Some(namespace) =
        resolve_explicit_package("@vue").filter(|path| is_vue_runtime_namespace(path))
    {
        return VueRuntimePackages::Namespace(namespace);
    }
    if let Some(runtime_dom) = resolve_explicit_package("@vue/runtime-dom") {
        return VueRuntimePackages::RuntimeDom(runtime_dom);
    }
    if let Some(namespace) = resolve_inferred_vue_namespace_package(project_root, vue_source) {
        return VueRuntimePackages::Namespace(namespace);
    }
    if let Some(runtime_dom) = resolve_inferred_package(project_root, "@vue/runtime-dom") {
        return VueRuntimePackages::RuntimeDom(runtime_dom);
    }
    VueRuntimePackages::Stub
}

fn resolve_inferred_vue_namespace_package(
    project_root: &Path,
    vue_source: &Path,
) -> Option<PathBuf> {
    let adjacent = resolve_adjacent_vue_namespace_package(vue_source);
    let ancestor = resolve_ancestor_package(project_root, "@vue");

    adjacent
        .filter(|path| is_vue_runtime_namespace(path))
        .or_else(|| ancestor.filter(|path| is_vue_runtime_namespace(path)))
        .or_else(|| {
            resolve_package_from_runtime_node_modules("@vue")
                .filter(|path| is_vue_runtime_namespace(path))
        })
        .or_else(|| {
            resolve_test_workspace_package("@vue").filter(|path| is_vue_runtime_namespace(path))
        })
}

fn resolve_adjacent_vue_namespace_package(vue_source: &Path) -> Option<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(parent) = vue_source.parent() {
        candidates.push(parent.join("@vue"));
    }

    if let Ok(real_vue_source) = std::fs::canonicalize(vue_source)
        && let Some(parent) = real_vue_source.parent()
    {
        candidates.push(parent.join("@vue"));
    }

    candidates
        .into_iter()
        .find(|candidate| candidate.exists() && is_vue_runtime_namespace(candidate))
}

fn is_vue_runtime_namespace(path: &Path) -> bool {
    path.join("runtime-dom").exists() || path.join("runtime-core").exists()
}

pub(super) fn resolve_vue_package(project_root: &Path) -> Option<PathBuf> {
    resolve_package(project_root, "vue")
}

pub(super) fn resolve_package(project_root: &Path, package: &str) -> Option<PathBuf> {
    resolve_explicit_package(package).or_else(|| resolve_inferred_package(project_root, package))
}

fn resolve_inferred_package(project_root: &Path, package: &str) -> Option<PathBuf> {
    resolve_ancestor_package(project_root, package)
        .or_else(|| resolve_package_from_runtime_node_modules(package))
        .or_else(|| resolve_test_workspace_package(package))
}

fn resolve_explicit_package(package: &str) -> Option<PathBuf> {
    explicit_package_env(package).and_then(resolve_explicit_package_env)
}

fn explicit_package_env(package: &str) -> Option<&'static str> {
    match package {
        "vue" => Some(VIZE_VUE_PACKAGE_ENV),
        "@vue" => Some(VIZE_VUE_NAMESPACE_PACKAGE_ENV),
        "@vue/runtime-dom" => Some(VIZE_VUE_RUNTIME_DOM_PACKAGE_ENV),
        "vite" => Some(VIZE_VITE_PACKAGE_ENV),
        _ => None,
    }
}

fn resolve_explicit_package_env(name: &str) -> Option<PathBuf> {
    env_var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.exists())
}

fn resolve_package_from_runtime_node_modules(package: &str) -> Option<PathBuf> {
    env_var_os(VIZE_RUNTIME_NODE_MODULES_ENV)
        .into_iter()
        .flat_map(|paths| env::split_paths(&paths).collect::<Vec<_>>())
        .map(|node_modules| node_modules.join(package_path(package)))
        .find(|candidate| candidate.exists())
}

fn package_path(package: &str) -> PathBuf {
    package.split('/').collect()
}

#[cfg(test)]
fn resolve_test_workspace_package(package: &str) -> Option<PathBuf> {
    if let Some(override_path) = env_var_os(VIZE_TEST_WORKSPACE_NODE_MODULES_ENV) {
        if override_path.as_os_str() == "__none__" {
            return None;
        }
        let candidate = PathBuf::from(override_path).join(package_path(package));
        return candidate.exists().then_some(candidate);
    }

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)?;
    let candidate = workspace_root
        .join("node_modules")
        .join(package_path(package));
    candidate.exists().then_some(candidate)
}

#[cfg(not(test))]
fn resolve_test_workspace_package(_package: &str) -> Option<PathBuf> {
    None
}

fn resolve_ancestor_package(project_root: &Path, package: &str) -> Option<PathBuf> {
    let mut current = Some(project_root);

    while let Some(dir) = current {
        let candidate = dir.join("node_modules").join(package_path(package));
        if candidate.exists() {
            return Some(candidate);
        }
        current = dir.parent();
    }

    None
}

#[cfg(test)]
mod test_override_tests {
    use std::sync::{Arc, Barrier};

    use super::*;

    #[test]
    fn resolver_overrides_are_isolated_to_the_calling_thread() {
        let actual = env::var_os(VIZE_VUE_PACKAGE_ENV);
        let override_path = PathBuf::from("thread-local-vue-package");
        let barrier = Arc::new(Barrier::new(2));

        std::thread::scope(|scope| {
            let worker_barrier = Arc::clone(&barrier);
            let worker_actual = actual.clone();
            let worker_override = override_path.clone();
            let worker = scope.spawn(move || {
                let observed_inside = with_test_env_overrides(
                    &[(VIZE_VUE_PACKAGE_ENV, Some(worker_override.as_path()))],
                    || {
                        worker_barrier.wait();
                        worker_barrier.wait();
                        test_env_var_os(VIZE_VUE_PACKAGE_ENV)
                    },
                );
                let observed_after = test_env_var_os(VIZE_VUE_PACKAGE_ENV);
                (observed_inside, observed_after, worker_actual)
            });

            barrier.wait();
            let observed_concurrently = test_env_var_os(VIZE_VUE_PACKAGE_ENV);
            barrier.wait();

            let (observed_inside, observed_after, worker_actual) = worker.join().unwrap();
            assert_eq!(observed_concurrently, actual);
            assert_eq!(observed_inside, Some(override_path.into_os_string()));
            assert_eq!(observed_after, worker_actual);
        });
    }

    #[test]
    fn nested_absent_override_restores_after_return_and_panic() {
        let actual = env::var_os(VIZE_VUE_PACKAGE_ENV);
        let outer_path = PathBuf::from("outer-vue-package");

        with_test_env_overrides(
            &[(VIZE_VUE_PACKAGE_ENV, Some(outer_path.as_path()))],
            || {
                assert_eq!(
                    test_env_var_os(VIZE_VUE_PACKAGE_ENV),
                    Some(outer_path.clone().into_os_string())
                );

                with_test_env_overrides(&[(VIZE_VUE_PACKAGE_ENV, None)], || {
                    assert_eq!(test_env_var_os(VIZE_VUE_PACKAGE_ENV), None);
                });
                assert_eq!(
                    test_env_var_os(VIZE_VUE_PACKAGE_ENV),
                    Some(outer_path.clone().into_os_string())
                );

                let panic = std::panic::catch_unwind(|| {
                    with_test_env_overrides(&[(VIZE_VUE_PACKAGE_ENV, None)], || {
                        assert_eq!(test_env_var_os(VIZE_VUE_PACKAGE_ENV), None);
                        panic!("test override cleanup");
                    });
                });
                assert!(panic.is_err());
                assert_eq!(
                    test_env_var_os(VIZE_VUE_PACKAGE_ENV),
                    Some(outer_path.clone().into_os_string())
                );
            },
        );

        assert_eq!(test_env_var_os(VIZE_VUE_PACKAGE_ENV), actual);
    }
}
