use std::path::{Path, PathBuf};

use super::error::CorsaResult;

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
export type LifecycleHook = () => void | Promise<void>;

export type InjectionKey<T> = symbol & { readonly __vize_injection?: T };

export interface ComponentPublicInstance {
  $attrs: any;
  $slots: any;
  $refs: any;
  $emit: (...args: any[]) => void;
}

export interface App<Element = any> {
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
export declare function provide<T>(key: InjectionKey<T> | string | symbol, value: T): void;
export declare function inject<T>(key: InjectionKey<T> | string | symbol): T | undefined;
export declare function inject<T>(key: InjectionKey<T> | string | symbol, defaultValue: T): T;
export declare function watch<T>(source: any, cb: any): WatchStopHandle;
export declare function watchEffect(effect: () => void | Promise<void>): WatchStopHandle;
export declare function onMounted(hook: LifecycleHook): void;
export declare function onUnmounted(hook: LifecycleHook): void;
export declare function onBeforeMount(hook: LifecycleHook): void;
export declare function onBeforeUnmount(hook: LifecycleHook): void;
export declare function onBeforeUpdate(hook: LifecycleHook): void;
export declare function onUpdated(hook: LifecycleHook): void;
export declare function nextTick<T>(fn: () => T | Promise<T>): Promise<T>;
export declare function nextTick(): Promise<void>;
export declare function useTemplateRef<T = any>(key: string): ShallowRef<T | null>;
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
    std::fs::create_dir_all(&node_modules_dir)?;

    materialize_vue_support(project_root, &node_modules_dir)?;
    materialize_vite_support(project_root, &node_modules_dir)?;

    Ok(())
}

fn materialize_vue_support(project_root: &Path, node_modules_dir: &Path) -> std::io::Result<()> {
    let vue_target = node_modules_dir.join("vue");
    let vue_namespace_target = node_modules_dir.join("@vue");

    if let (Some(vue_source), Some(vue_namespace_source)) = (
        resolve_ancestor_package(project_root, "vue"),
        resolve_ancestor_package(project_root, "@vue"),
    ) {
        if symlink_path(&vue_source, &vue_target).is_ok()
            && symlink_path(&vue_namespace_source, &vue_namespace_target).is_ok()
        {
            return Ok(());
        }
    }

    remove_path(&vue_namespace_target)?;
    write_vue_stub(node_modules_dir)
}

fn materialize_vite_support(project_root: &Path, node_modules_dir: &Path) -> std::io::Result<()> {
    let vite_target = node_modules_dir.join("vite");

    if let Some(vite_source) = resolve_ancestor_package(project_root, "vite") {
        if symlink_path(&vite_source, &vite_target).is_ok() {
            return Ok(());
        }
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
    remove_path(&vue_dir)?;
    std::fs::create_dir_all(&vue_dir)?;
    std::fs::write(vue_dir.join("package.json"), VUE_STUB_PACKAGE_JSON)?;
    std::fs::write(vue_dir.join("index.d.ts"), VUE_STUB_TYPES)?;
    Ok(())
}

fn write_vite_stub(node_modules_dir: &Path) -> std::io::Result<()> {
    let vite_dir = node_modules_dir.join("vite");
    remove_path(&vite_dir)?;
    std::fs::create_dir_all(&vite_dir)?;
    std::fs::write(vite_dir.join("package.json"), VITE_STUB_PACKAGE_JSON)?;
    std::fs::write(vite_dir.join("client.d.ts"), VITE_CLIENT_STUB)?;
    Ok(())
}

fn remove_path(path: &Path) -> std::io::Result<()> {
    if path.is_symlink() || path.is_file() {
        std::fs::remove_file(path)?;
    } else if path.exists() {
        std::fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn symlink_path(source: &Path, target: &Path) -> std::io::Result<()> {
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent)?;
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
