/**
 * Mock Nuxt runtime composables.
 */

import { reactive, ref } from "vue";
import type { Ref } from "vue";
import type { NuxtMuseaOptions } from "../types.js";

let _runtimeConfig: NuxtMuseaOptions["runtimeConfig"] = {};
let _stateMocks: Record<string, unknown> = {};

export function _setRuntimeConfig(config: NuxtMuseaOptions["runtimeConfig"]): void {
  _runtimeConfig = config;
}

export function _setStateMocks(mocks: Record<string, unknown>): void {
  _stateMocks = mocks;
}

/**
 * Mock useNuxtApp - returns a minimal Nuxt app-like object.
 */
export function useNuxtApp() {
  return {
    $config: reactive({
      public: _runtimeConfig?.public ?? {},
      ..._runtimeConfig,
    }),
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
  return reactive({
    public: _runtimeConfig?.public ?? {},
    ..._runtimeConfig,
  });
}

/**
 * Mock useState - returns a ref initialized from mock config or init function.
 */
export function useState<T = unknown>(key: string, init?: () => T): Ref<T | undefined> {
  if (key in _stateMocks) {
    return ref(_stateMocks[key] as T);
  }
  return ref(init ? init() : undefined) as Ref<T | undefined>;
}

/**
 * Mock useRequestHeaders - returns empty headers in gallery context.
 */
export function useRequestHeaders(_include?: string[]): Record<string, string> {
  return {};
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
  return new URL("http://localhost:3000");
}

/**
 * Mock useCookie - returns a ref-like cookie mock.
 */
export function useCookie<T = unknown>(
  _name: string,
  _opts?: Record<string, unknown>,
): Ref<T | undefined> {
  const value = ref<T | undefined>(undefined);
  return value;
}

/**
 * Mock clearNuxtState - no-op.
 */
export function clearNuxtState(_keys?: string | string[]): void {
  // no-op
}

/**
 * Mock defineNuxtPlugin - returns the plugin function as-is.
 */
export function defineNuxtPlugin(plugin: unknown): unknown {
  return plugin;
}
