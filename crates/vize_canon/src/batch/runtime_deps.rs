use std::path::{Path, PathBuf};

use super::error::CorsaResult;
use super::materialize_fs::{ensure_dir, prune_dir_entries, remove_path, write_if_changed};
use vize_carton::FxHashSet;

const VUE_STUB_PACKAGE_JSON: &str = r#"{
  "name": "vue",
  "types": "index.d.ts"
}
"#;

const VUE_STUB_TYPES: &str = r#"export interface Ref<T = any, S = T> {
  value: T;
}

export interface ShallowRef<T = any, S = T> extends Ref<T, S> {}

export interface ComputedRef<T = any> extends Readonly<Ref<T>> {
  readonly value: T;
}

export type UnwrapRef<T> = T extends Ref<infer V, any> ? V : T;
export type WatchStopHandle = () => void;
export type WatchCleanup = (cleanupFn: () => void) => void;
export type WatchEffect = (onCleanup: WatchCleanup) => void | Promise<void>;
export type LifecycleHook = () => void | Promise<void>;

export type InjectionKey<T> = symbol & { readonly __vize_injection?: T };
export type PropConstructor<T = any> =
  | { new (...args: any[]): T & {} }
  | { (): T }
  | ((...args: any[]) => T);
export type PropType<T> = PropConstructor<T> | readonly PropConstructor<T>[];

export interface ComponentCustomProperties {}

export interface ComponentPublicInstance extends ComponentCustomProperties {
  $attrs: any;
  $slots: any;
  $refs: any;
  $emit: (...args: any[]) => void;
}

export interface App<Element = any> {
  config: {
    globalProperties: ComponentCustomProperties & Record<string, any>;
    [key: string]: any;
  };
  mount(rootContainer: string | Element): ComponentPublicInstance;
  unmount(): void;
  use(plugin: any, ...options: any[]): App<Element>;
  provide<T>(key: InjectionKey<T> | string | symbol, value: T): App<Element>;
  component(name: string, component?: any): any;
}

export type DefineComponent<
  Props = any,
  _RawBindings = any,
  _Data = any,
  _Computed = any,
  _Methods = any,
  _Mixin = any,
  _Extends = any,
  Emits = any,
> = new (...args: any[]) => ComponentPublicInstance & {
  $props: Props;
  $emit: Emits extends (...args: any[]) => any ? Emits : (...args: any[]) => void;
};

export declare function ref<T>(value: T): Ref<T>;
export declare function shallowRef<T>(value: T): ShallowRef<T>;
export declare function computed<T>(getter: () => T): ComputedRef<T>;
export declare function reactive<T extends object>(value: T): T;
export declare function readonly<T>(value: T): Readonly<T>;
export declare function createApp(rootComponent: any, rootProps?: any): App;
export declare function createSSRApp(rootComponent: any, rootProps?: any): App;
export declare function defineComponent<Props = any>(options: any): DefineComponent<Props>;
export declare function defineProps<T = any>(): T;
export declare function defineProps<const T extends readonly string[]>(_props: T): { [K in T[number]]?: any };
export declare function defineProps<const T extends Record<string, any>>(_props: T): T;
export declare function provide<T>(key: InjectionKey<T> | string | symbol, value: T): void;
export declare function inject<T>(key: InjectionKey<T> | string | symbol): T | undefined;
export declare function inject<T>(key: InjectionKey<T> | string | symbol, defaultValue: T): T;
export declare function watch<T>(source: any, cb: any, options?: any): WatchStopHandle;
export declare function watchEffect(effect: WatchEffect, options?: any): WatchStopHandle;
export declare function onMounted(hook: LifecycleHook): void;
export declare function onUnmounted(hook: LifecycleHook): void;
export declare function onBeforeMount(hook: LifecycleHook): void;
export declare function onBeforeUnmount(hook: LifecycleHook): void;
export declare function onBeforeUpdate(hook: LifecycleHook): void;
export declare function onUpdated(hook: LifecycleHook): void;
export declare function nextTick<T>(fn: () => T | Promise<T>): Promise<T>;
export declare function nextTick(): Promise<void>;
export declare function useTemplateRef<T = any>(key: string): ShallowRef<T | null>;
export declare function useId(): string;
export declare const Transition: DefineComponent;
export declare const TransitionGroup: DefineComponent;
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

    if let (Some(vue_source), Some(vue_namespace_source)) = (
        resolve_ancestor_package(project_root, "vue"),
        resolve_ancestor_package(project_root, "@vue"),
    ) && symlink_path(&vue_source, &vue_target).is_ok()
        && symlink_path(&vue_namespace_source, &vue_namespace_target).is_ok()
    {
        return Ok(());
    }

    remove_path(&vue_namespace_target)?;
    write_vue_stub(node_modules_dir)
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

fn resolve_ancestor_package(project_root: &Path, package: &str) -> Option<PathBuf> {
    let mut current = Some(project_root);

    while let Some(dir) = current {
        let candidate = dir.join("node_modules").join(package);
        if candidate.exists() {
            return Some(candidate);
        }
        current = dir.parent();
    }

    None
}

fn write_vue_stub(node_modules_dir: &Path) -> std::io::Result<()> {
    let vue_dir = node_modules_dir.join("vue");
    ensure_stub_dir(&vue_dir)?;
    write_if_changed(
        &vue_dir.join("package.json"),
        VUE_STUB_PACKAGE_JSON.as_bytes(),
    )?;
    write_if_changed(&vue_dir.join("index.d.ts"), VUE_STUB_TYPES.as_bytes())?;
    prune_stub_dir(&vue_dir, &["package.json", "index.d.ts"])?;
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
