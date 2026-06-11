//! Built-in fallback stubs and `nuxt.config` module-driven fallbacks.

use std::path::Path;

use vize_carton::{FxHashSet, String, ToCompactString};

use super::stubs::{
    declared_name, push_generic_function_stub, push_named_overload_stubs, push_stub,
    tracked_read_to_string,
};

pub(super) fn collect_fallback_stubs(
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    has_generated_imports: bool,
) {
    let mut fallback_names = FxHashSet::default();
    push_fallback_stub_group(
        fallback_type_alias_stubs(),
        stubs,
        seen_names,
        &mut fallback_names,
    );
    // The hardcoded `any` ladder silently weakens checking, so only inject it
    // when the project has no generated `.nuxt` import manifest to rely on.
    if !has_generated_imports {
        push_fallback_stub_group(
            fallback_value_stubs(),
            stubs,
            seen_names,
            &mut fallback_names,
        );
    }
    // Built-in component consts stay name-deduped against the generated
    // `GlobalComponents` members: `.nuxt/components.d.ts` may omit built-ins,
    // and dropping these would newly error every `<NuxtLink>` reference.
    push_fallback_stub_group(
        fallback_component_stubs(),
        stubs,
        seen_names,
        &mut fallback_names,
    );
}

fn push_fallback_stub_group(
    group: Vec<String>,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    fallback_names: &mut FxHashSet<String>,
) {
    for stub in group {
        if let Some(name) = declared_name(&stub) {
            let name = name.to_compact_string();
            if fallback_names.contains(name.as_str()) {
                stubs.push(stub);
                continue;
            }
            if seen_names.insert(name.clone()) {
                fallback_names.insert(name);
                stubs.push(stub);
            }
            continue;
        }
        stubs.push(stub);
    }
}

pub(super) fn collect_module_fallback_stubs(
    cwd: &Path,
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
) {
    let config_source = nuxt_config_source(cwd);
    if config_source.is_empty() {
        return;
    }

    if config_source.contains("@nuxtjs/i18n") || config_source.contains("@nuxt/i18n") {
        push_stub(
            stubs,
            seen_names,
            "declare function useI18n(): ({ locale: { value: string }; locales: (Array<{ code: string; name?: string; dir?: any }> & { value: Array<{ code: string; name?: string; dir?: any }> }); t: (...args: any[]) => any } & Record<string, any>);"
                .into(),
        );
        push_generic_function_stub(stubs, seen_names, "useLocalePath");
        push_generic_function_stub(stubs, seen_names, "$t");
        stubs.push("declare module \"@nuxtjs/i18n\" { export type Directions = any; }".into());
    }

    if config_source.contains("@vueuse/nuxt") {
        for name in ["useClipboard", "usePreferredDark", "useScrollLock"] {
            push_generic_function_stub(stubs, seen_names, name);
        }
        push_named_overload_stubs(
            stubs,
            seen_names,
            "onKeyStroke",
            vec![
                "declare function onKeyStroke(key: string | string[], handler: (event: KeyboardEvent) => any, options?: any): void;".into(),
                "declare function onKeyStroke(predicate: (event: KeyboardEvent) => any, handler: (event: KeyboardEvent) => any, options?: any): void;".into(),
                "declare function onKeyStroke(handler: (event: KeyboardEvent) => any, options?: any): void;".into(),
            ],
        );
        stubs.push(
            "declare module \"@vueuse/integrations/useFocusTrap\" { export function useFocusTrap(...args: any[]): any; }"
                .into(),
        );
    }

    if config_source.contains("@nuxtjs/color-mode") {
        push_stub(
            stubs,
            seen_names,
            "declare function useColorMode(): ({ preference: 'system' | 'light' | 'dark'; value: any } & Record<string, any>);"
                .into(),
        );
    }

    if config_source.contains("nuxt-og-image") {
        push_generic_function_stub(stubs, seen_names, "defineOgImageComponent");
    }
}

pub(super) fn nuxt_config_source(cwd: &Path) -> String {
    for file_name in ["nuxt.config.ts", "nuxt.config.js", "nuxt.config.mts"] {
        let path = cwd.join(file_name);
        if let Ok(source) = tracked_read_to_string(path.as_path()) {
            return source.into();
        }
    }
    String::default()
}

#[cfg(test)]
pub(super) fn fallback_stub_strings() -> Vec<String> {
    let mut stubs = fallback_type_alias_stubs();
    stubs.extend(fallback_value_stubs());
    stubs.extend(fallback_component_stubs());
    stubs
}

/// Type aliases for auto-imported Vue types. Generated `.nuxt` artifacts are
/// only mined for `declare global` consts and `GlobalComponents` members, so
/// these aliases have no generated counterpart and are always injected.
fn fallback_type_alias_stubs() -> Vec<String> {
    vec![
        "type Composer = any;".into(),
        "type Ref<T = any> = import('vue').Ref<T>;".into(),
        "type ComputedRef<T = any> = import('vue').ComputedRef<T>;".into(),
        "type WritableComputedRef<T = any> = import('vue').WritableComputedRef<T>;".into(),
        "type ShallowRef<T = any> = import('vue').ShallowRef<T>;".into(),
        "type UnwrapRef<T> = import('vue').UnwrapRef<T>;".into(),
        "type UnwrapNestedRefs<T> = import('vue').UnwrapNestedRefs<T>;".into(),
        "type MaybeRef<T = any> = import('vue').MaybeRef<T>;".into(),
        "type MaybeRefOrGetter<T = any> = import('vue').MaybeRefOrGetter<T>;".into(),
        "type Component = import('vue').Component;".into(),
    ]
}

/// Hardcoded `any`-typed value stubs. Skipped whenever the project ships a
/// generated `.nuxt` import manifest, which covers all of these names with
/// real `typeof import(...)` types.
fn fallback_value_stubs() -> Vec<String> {
    vec![
        "declare function ref<T>(value: T): Ref<UnwrapRef<T>>;".into(),
        "declare function ref<T = any>(): Ref<T | undefined>;".into(),
        "declare function computed<T>(getter: () => T): ComputedRef<T>;".into(),
        "declare function computed<T>(options: { get: () => T; set: (value: T) => void }): WritableComputedRef<T>;".into(),
        "declare function reactive<T extends object>(target: T): UnwrapNestedRefs<T>;".into(),
        "declare function readonly<T extends object>(target: T): Readonly<T>;".into(),
        "declare function watch(source: any, cb: (...args: any[]) => any, options?: any): any;".into(),
        "declare function watchEffect(effect: () => void, options?: any): any;".into(),
        "declare function watchPostEffect(effect: () => void): any;".into(),
        "declare function watchSyncEffect(effect: () => void): any;".into(),
        "declare function onMounted(hook: () => any): void;".into(),
        "declare function onUnmounted(hook: () => any): void;".into(),
        "declare function onBeforeMount(hook: () => any): void;".into(),
        "declare function onBeforeUnmount(hook: () => any): void;".into(),
        "declare function onBeforeUpdate(hook: () => any): void;".into(),
        "declare function onUpdated(hook: () => any): void;".into(),
        "declare function onActivated(hook: () => any): void;".into(),
        "declare function onDeactivated(hook: () => any): void;".into(),
        "declare function onErrorCaptured(hook: (...args: any[]) => any): void;".into(),
        "declare function nextTick(fn?: () => void): Promise<void>;".into(),
        "declare function toRef<T extends object, K extends keyof T>(object: T, key: K): Ref<T[K]>;".into(),
        "declare function toRefs<T extends object>(object: T): { [K in keyof T]: Ref<T[K]> };".into(),
        "declare function unref<T>(ref: T | Ref<T>): T;".into(),
        "declare function isRef(value: any): value is Ref;".into(),
        "declare function shallowRef<T>(value: T): ShallowRef<T>;".into(),
        "declare function triggerRef(ref: ShallowRef): void;".into(),
        "declare function provide<T>(key: string | symbol, value: T): void;".into(),
        "declare function inject<T>(key: string | symbol): T | undefined;".into(),
        "declare function inject<T>(key: string | symbol, defaultValue: T): T;".into(),
        "declare function defineAsyncComponent(source: any): any;".into(),
        "declare function h(type: any, ...args: any[]): any;".into(),
        "declare function useAttrs(): Record<string, unknown>;".into(),
        "declare function useSlots(): Record<string, (...args: any[]) => any>;".into(),
        "declare function toRaw<T>(observed: T): T;".into(),
        "declare function markRaw<T extends object>(value: T): T;".into(),
        "declare function effectScope(detached?: boolean): any;".into(),
        "declare function getCurrentScope(): any;".into(),
        "declare function onScopeDispose(fn: () => void): void;".into(),
        "declare function shallowReactive<T extends object>(target: T): T;".into(),
        "declare function shallowReadonly<T extends object>(target: T): Readonly<T>;".into(),
        "declare function customRef<T>(factory: any): Ref<T>;".into(),
        "declare function useRouter(): any;".into(),
        "declare function useRoute(name?: string): any;".into(),
        "declare function definePageMeta(meta: any): void;".into(),
        "declare function defineRouteRules(rules: any): void;".into(),
        "declare function useSeoMeta(meta: any): void;".into(),
        "declare function useFetch<T = any>(url: string | (() => string), options?: any): any;".into(),
        "declare function useAsyncData<T = any>(handler: (...args: any[]) => T | Promise<T>, options?: any): any;".into(),
        "declare function useAsyncData<T = any>(key: string, handler: (...args: any[]) => T | Promise<T>, options?: any): any;".into(),
        "declare function useLazyFetch<T = any>(url: string | (() => string), options?: any): any;".into(),
        "declare function useLazyAsyncData<T = any>(handler: (...args: any[]) => T | Promise<T>, options?: any): any;".into(),
        "declare function useLazyAsyncData<T = any>(key: string, handler: (...args: any[]) => T | Promise<T>, options?: any): any;".into(),
        "declare function navigateTo(to: string | any, options?: any): any;".into(),
        "declare function createError(input: string | { statusCode?: number; statusMessage?: string; message?: string; data?: any; fatal?: boolean }): any;".into(),
        "declare function showError(error: any): any;".into(),
        "declare function clearError(options?: { redirect?: string }): Promise<void>;".into(),
        "declare function useNuxtApp(): any;".into(),
        "declare function useRuntimeConfig(): any;".into(),
        "declare function useAppConfig(): any;".into(),
        "declare function useState<T = any>(key: string, init?: () => T): Ref<T>;".into(),
        "declare function useCookie<T = any>(name: string, options?: any): Ref<T>;".into(),
        "declare function useHead(input: { titleTemplate?: (titleChunk?: string) => any; [key: string]: any }): void;".into(),
        "declare function useHead(input: any): void;".into(),
        "declare function useRequestHeaders(headers?: string[]): Record<string, string>;".into(),
        "declare function useRequestURL(): URL;".into(),
        "declare function defineNuxtComponent(options: any): any;".into(),
        "declare function defineNuxtRouteMiddleware(middleware: any): any;".into(),
        "declare function useError(): any;".into(),
        "declare function abortNavigation(err?: any): any;".into(),
        "declare function addRouteMiddleware(name: string, middleware: any, options?: any): void;".into(),
        "declare function defineNuxtPlugin(plugin: any): any;".into(),
        "declare function setPageLayout(layout: string): void;".into(),
        "declare function setResponseStatus(code: number, message?: string): void;".into(),
        "declare function prerenderRoutes(routes: string | string[]): void;".into(),
        "declare function refreshNuxtData(keys?: string | string[]): Promise<void>;".into(),
        "declare function clearNuxtData(keys?: string | string[]): void;".into(),
        "declare function reloadNuxtApp(options?: any): void;".into(),
        "declare function callOnce(key: string, fn: () => any): Promise<void>;".into(),
        "declare function callOnce(fn: () => any): Promise<void>;".into(),
        "declare function onNuxtReady(callback: () => any): void;".into(),
        "declare function preloadComponents(components: string | string[]): Promise<void>;".into(),
        "declare function prefetchComponents(components: string | string[]): Promise<void>;".into(),
        "declare function useRequestEvent(): any;".into(),
        "declare function useRequestFetch(): typeof globalThis.fetch;".into(),
        "declare function useResponseHeaders(headers?: Record<string, string>): any;".into(),
        "declare function $fetch<T = any>(...args: any[]): Promise<T>;".into(),
    ]
}

/// Nuxt built-in component consts. Always injected (name-deduped against the
/// generated `GlobalComponents` members) because `.nuxt/components.d.ts` may
/// legitimately omit built-ins.
fn fallback_component_stubs() -> Vec<String> {
    vec![
        "declare const NuxtLink: any;".into(),
        "declare const NuxtPage: any;".into(),
        "declare const NuxtLayout: any;".into(),
        "declare const NuxtLoadingIndicator: any;".into(),
        "declare const NuxtErrorBoundary: any;".into(),
        "declare const NuxtWelcome: any;".into(),
        "declare const NuxtIsland: any;".into(),
        "declare const NuxtRouteAnnouncer: any;".into(),
        "declare const ClientOnly: any;".into(),
        "declare const DevOnly: any;".into(),
    ]
}
