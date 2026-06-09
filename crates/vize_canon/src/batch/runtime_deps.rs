use std::{
    env,
    path::{Path, PathBuf},
};

use super::error::CorsaResult;
use super::materialize_fs::{ensure_dir, prune_dir_entries, remove_path, write_if_changed};
use vize_carton::FxHashSet;

const VIZE_VUE_PACKAGE_ENV: &str = "VIZE_VUE_PACKAGE";
const VIZE_RUNTIME_NODE_MODULES_ENV: &str = "VIZE_RUNTIME_NODE_MODULES";
#[cfg(test)]
const VIZE_TEST_WORKSPACE_NODE_MODULES_ENV: &str = "VIZE_TEST_WORKSPACE_NODE_MODULES";

const VUE_FACADE_PACKAGE_JSON: &str = r#"{
  "name": "vue",
  "types": "index.d.ts"
}
"#;

const VUE_FACADE_TYPES: &str = r#"export * from "@vue/runtime-dom";
"#;

const VUE_RUNTIME_DOM_STUB_PACKAGE_JSON: &str = r#"{
  "name": "@vue/runtime-dom",
  "types": "index.d.ts"
}
"#;

const VUE_RUNTIME_DOM_STUB_TYPES: &str = r#"export interface ComponentPublicInstance<Props = {}> {
  $props: Props;
  $attrs: { [key: string]: unknown };
  $slots: { [key: string]: unknown };
  $refs: { [key: string]: unknown };
  $emit: (...args: any[]) => void;
}

export type DefineComponent<
  Props = {},
  RawBindings = {},
  D = {},
  C = {},
  M = {},
  Mixin = {},
  Extends = {},
  E = {},
  EE = string,
  PP = Props,
  PropsDefaults = {},
  MakeDefaultsOptional = true,
  Options = {},
  S = {}
> = {
  new (): ComponentPublicInstance<Props>;
};

export interface Ref<T = unknown, _Raw = T> {
  value: T;
}

export interface ComputedRef<T = unknown> extends Ref<T> {
  readonly value: T;
}

export interface WritableComputedRef<T = unknown> extends Ref<T> {
  value: T;
}

export interface ShallowRef<T = unknown, _Raw = T> extends Ref<T, _Raw> {
  readonly __v_isShallow?: true;
}

export type InjectionKey<T> = symbol & { readonly __v_vlsInjection?: T };
export type PropType<T> = { new (...args: any[]): T & {} } | { (): T } | null;

export declare const Transition: DefineComponent;
export declare function defineComponent(options: any): DefineComponent;
export declare function defineAsyncComponent(source: any): DefineComponent;
export declare function defineProps<T = {}>(): T;
export declare function computed<T>(getter: () => T): ComputedRef<T>;
export declare function computed<T>(options: { get: () => T; set: (value: T) => void }): WritableComputedRef<T>;
export declare function ref<T>(value: T): Ref<T>;
export declare function reactive<T extends object>(target: T): T;
export declare function shallowRef<T>(value: T): ShallowRef<T>;
export declare function toRef<T extends object, K extends keyof T>(object: T, key: K): Ref<T[K]>;
export declare function useTemplateRef<T = unknown>(key: string): ShallowRef<T | null>;
export declare function useId(): string;
export declare function watch<T>(source: T, callback: (...args: any[]) => void, options?: any): void;
export declare function watchEffect(effect: (onCleanup: (cleanupFn: () => void) => void) => void): void;
export declare function onMounted(callback: () => void): void;
export declare function customRef<T>(factory: any): Ref<T>;
export declare function provide<T>(key: InjectionKey<T> | string | symbol, value: T): void;
export declare function inject<T>(key: InjectionKey<T> | string | symbol): T | undefined;
export declare function inject<T>(key: InjectionKey<T> | string | symbol, defaultValue: T): T;
export declare function markRaw<T extends object>(value: T): T;
export declare function createApp(root: any): {
  config: {
    globalProperties: { [key: string]: any };
  };
};
"#;

const VITE_STUB_PACKAGE_JSON: &str = r#"{
  "name": "vite",
  "types": "client.d.ts"
}
"#;

const VITE_CLIENT_STUB: &str = r#"interface ImportMetaEnv {
  readonly [key: string]: string | boolean | undefined;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

export {};
"#;

pub(super) fn materialize_runtime_dependencies(
    project_root: &Path,
    virtual_root: &Path,
) -> CorsaResult<()> {
    let node_modules_dir = virtual_root.join("node_modules");
    ensure_dir(&node_modules_dir)?;

    materialize_vue_support(project_root, &node_modules_dir)?;
    materialize_vite_support(project_root, &node_modules_dir)?;
    prune_runtime_node_modules(&node_modules_dir)?;

    Ok(())
}

fn materialize_vue_support(project_root: &Path, node_modules_dir: &Path) -> std::io::Result<()> {
    let vue_target = node_modules_dir.join("vue");
    let vue_namespace_target = node_modules_dir.join("@vue");

    if let Some(vue_source) = resolve_vue_package(project_root)
        && symlink_path(&package_link_source(&vue_source), &vue_target).is_ok()
    {
        if let Some(vue_namespace_source) = resolve_vue_namespace_package(project_root, &vue_source)
        {
            if symlink_path(&vue_namespace_source, &vue_namespace_target).is_err() {
                remove_path(&vue_namespace_target)?;
            }
        } else if let Some(runtime_dom_source) = resolve_package(project_root, "@vue/runtime-dom") {
            link_vue_runtime_dom_package(node_modules_dir, &runtime_dom_source)?;
        } else {
            write_vue_runtime_dom_stub(node_modules_dir)?;
        }
        return Ok(());
    }

    if let Some(runtime_dom_source) = resolve_package(project_root, "@vue/runtime-dom") {
        write_vue_facade(node_modules_dir)?;
        link_vue_runtime_dom_package(node_modules_dir, &runtime_dom_source)?;
        return Ok(());
    }

    write_vue_facade(node_modules_dir)?;
    write_vue_runtime_dom_stub(node_modules_dir)?;
    Ok(())
}

fn materialize_vite_support(project_root: &Path, node_modules_dir: &Path) -> std::io::Result<()> {
    let vite_target = node_modules_dir.join("vite");

    if let Some(vite_source) = resolve_ancestor_package(project_root, "vite")
        && symlink_path(&vite_source, &vite_target).is_ok()
    {
        return Ok(());
    }

    write_vite_stub(node_modules_dir)
}

fn resolve_vue_namespace_package(project_root: &Path, vue_source: &Path) -> Option<PathBuf> {
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

fn resolve_vue_package(project_root: &Path) -> Option<PathBuf> {
    resolve_ancestor_package(project_root, "vue")
        .or_else(|| resolve_explicit_package_env(VIZE_VUE_PACKAGE_ENV))
        .or_else(|| resolve_package_from_runtime_node_modules("vue"))
        .or_else(|| resolve_test_workspace_package("vue"))
}

fn resolve_package(project_root: &Path, package: &str) -> Option<PathBuf> {
    resolve_ancestor_package(project_root, package)
        .or_else(|| resolve_package_from_runtime_node_modules(package))
        .or_else(|| resolve_test_workspace_package(package))
}

fn package_link_source(source: &Path) -> PathBuf {
    std::fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf())
}

fn resolve_explicit_package_env(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.exists())
}

fn resolve_package_from_runtime_node_modules(package: &str) -> Option<PathBuf> {
    env::var_os(VIZE_RUNTIME_NODE_MODULES_ENV)
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
    if let Some(override_path) = env::var_os(VIZE_TEST_WORKSPACE_NODE_MODULES_ENV) {
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

fn write_vue_facade(node_modules_dir: &Path) -> std::io::Result<()> {
    let vue_dir = node_modules_dir.join("vue");
    ensure_stub_dir(&vue_dir)?;
    write_if_changed(
        &vue_dir.join("package.json"),
        VUE_FACADE_PACKAGE_JSON.as_bytes(),
    )?;
    write_if_changed(&vue_dir.join("index.d.ts"), VUE_FACADE_TYPES.as_bytes())?;
    prune_stub_dir(&vue_dir, &["package.json", "index.d.ts"])?;
    Ok(())
}

fn link_vue_runtime_dom_package(
    node_modules_dir: &Path,
    runtime_dom_source: &Path,
) -> std::io::Result<()> {
    let vue_namespace_dir = node_modules_dir.join("@vue");
    ensure_stub_dir(&vue_namespace_dir)?;
    let runtime_dom_target = vue_namespace_dir.join("runtime-dom");
    symlink_path(
        &package_link_source(runtime_dom_source),
        &runtime_dom_target,
    )
}

fn write_vue_runtime_dom_stub(node_modules_dir: &Path) -> std::io::Result<()> {
    let vue_namespace_dir = node_modules_dir.join("@vue");
    ensure_stub_dir(&vue_namespace_dir)?;
    let runtime_dom_dir = vue_namespace_dir.join("runtime-dom");
    ensure_stub_dir(&runtime_dom_dir)?;
    write_if_changed(
        &runtime_dom_dir.join("package.json"),
        VUE_RUNTIME_DOM_STUB_PACKAGE_JSON.as_bytes(),
    )?;
    write_if_changed(
        &runtime_dom_dir.join("index.d.ts"),
        VUE_RUNTIME_DOM_STUB_TYPES.as_bytes(),
    )?;
    prune_stub_dir(&runtime_dom_dir, &["package.json", "index.d.ts"])?;
    Ok(())
}

fn write_vite_stub(node_modules_dir: &Path) -> std::io::Result<()> {
    let vite_dir = node_modules_dir.join("vite");
    ensure_stub_dir(&vite_dir)?;
    write_if_changed(
        &vite_dir.join("package.json"),
        VITE_STUB_PACKAGE_JSON.as_bytes(),
    )?;
    write_if_changed(&vite_dir.join("client.d.ts"), VITE_CLIENT_STUB.as_bytes())?;
    prune_stub_dir(&vite_dir, &["package.json", "client.d.ts"])?;
    Ok(())
}

fn ensure_stub_dir(path: &Path) -> std::io::Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {}
        Ok(_) => {
            remove_path(path)?;
            ensure_dir(path)?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ensure_dir(path)?,
        Err(error) => return Err(error),
    }
    Ok(())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    if symlink_matches(source, target)? {
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        ensure_dir(parent)?;
    }

    remove_path(target)?;

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }

    #[cfg(windows)]
    {
        if source.is_dir() {
            std::os::windows::fs::symlink_dir(source, target)
        } else {
            std::os::windows::fs::symlink_file(source, target)
        }
    }
}

fn symlink_matches(source: &Path, target: &Path) -> std::io::Result<bool> {
    let metadata = match std::fs::symlink_metadata(target) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if !metadata.file_type().is_symlink() {
        return Ok(false);
    }
    let linked = std::fs::read_link(target)?;
    Ok(linked == source)
}

fn prune_stub_dir(dir: &Path, file_names: &[&str]) -> std::io::Result<()> {
    let expected_files = file_names
        .iter()
        .map(|name| dir.join(name))
        .collect::<FxHashSet<_>>();
    prune_dir_entries(dir, &expected_files)
}

fn prune_runtime_node_modules(node_modules_dir: &Path) -> std::io::Result<()> {
    let expected_files = FxHashSet::default();
    let preserved_roots = ["vue", "vite", "@vue"]
        .into_iter()
        .map(|name| node_modules_dir.join(name))
        .filter(|path| path.exists() || path.is_symlink())
        .collect::<Vec<_>>();
    super::materialize_fs::prune_unexpected_entries(
        node_modules_dir,
        &expected_files,
        &preserved_roots,
    )
}
