/**
 * Mock Nuxt runtime composables.
 */

import { reactive } from "vue";
import type { Ref } from "vue";

import {
  getAppConfigState,
  getCookieRef,
  getRequestState,
  getRuntimeConfigState,
  getStateRef,
  nextNuxtId,
  runCallOnce,
  setRuntimeConfig,
  setStateMocks,
  clearStateRefs,
  updateAppConfigState,
} from "../context.js";
import { useRoute, useRouter } from "./composables.js";

export { setRuntimeConfig as _setRuntimeConfig, setStateMocks as _setStateMocks };

/**
 * Mock useNuxtApp - returns a minimal Nuxt app-like object.
 */
export function useNuxtApp() {
  return {
    $config: useRuntimeConfig(),
    $router: useRouter(),
    $route: useRoute(),
    provide: (_name: string, _value: unknown) => {},
    hook: (_name: string, _fn: (...args: unknown[]) => void) => {},
    callHook: async (_name: string, ..._args: unknown[]) => {},
    vueApp: null,
    payload: reactive({ data: {}, state: {} }),
    isHydrating: false,
    runWithContext: <T>(fn: () => T) => fn(),
  };
}

/**
 * Mock useRuntimeConfig - returns the configured runtime config.
 */
export function useRuntimeConfig() {
  return getRuntimeConfigState();
}

export function useAppConfig() {
  return getAppConfigState();
}

export function updateAppConfig(config: Record<string, unknown>): void {
  updateAppConfigState(config);
}

/**
 * Mock useState - returns a ref initialized from mock config or init function.
 */
export function useState<T = unknown>(key: string, init?: () => T): Ref<T | undefined> {
  return getStateRef(key, init);
}

/**
 * Mock useRequestHeaders - returns empty headers in gallery context.
 */
export function useRequestHeaders(_include?: string[]): Record<string, string> {
  return { ...getRequestState().headers };
}

/**
 * Mock useRequestEvent - returns undefined in gallery context.
 */
export function useRequestEvent() {
  return undefined;
}

/**
 * Mock useRequestURL - returns current window location.
 */
export function useRequestURL(): URL {
  if (typeof window !== "undefined") {
    return new URL(window.location.href);
  }
  return new URL(getRequestState().url);
}

/**
 * Mock useCookie - returns a ref-like cookie mock.
 */
export function useCookie<T = unknown>(
  name: string,
  opts?: { default?: () => T },
): Ref<T | undefined> {
  return getCookieRef(name, opts?.default);
}

/**
 * Mock clearNuxtState - no-op.
 */
export function clearNuxtState(keys?: string | string[]): void {
  clearStateRefs(keys);
}

/**
 * Mock defineNuxtPlugin - returns the plugin function as-is.
 */
export function defineNuxtPlugin(plugin: unknown): unknown {
  return plugin;
}

export function defineNuxtComponent<T>(component: T): T {
  return component;
}

export function onNuxtReady(callback: () => void): void {
  if (typeof queueMicrotask === "function") {
    queueMicrotask(callback);
    return;
  }
  void Promise.resolve().then(callback);
}

export function callOnce<T>(
  keyOrFn: string | (() => T | Promise<T>),
  fn?: () => T | Promise<T>,
): Promise<T | undefined> {
  if (typeof keyOrFn === "function") {
    return runCallOnce("__default__", keyOrFn);
  }
  return runCallOnce(keyOrFn, fn ?? (() => undefined as T));
}

export function useId(): string {
  return nextNuxtId();
}

export function reloadNuxtApp(_options?: Record<string, unknown>): Promise<void> {
  return Promise.resolve();
}
