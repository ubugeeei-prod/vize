/**
 * Mock Nuxt routing composables.
 */

import { reactive, computed } from "vue";
import type { NuxtMuseaOptions } from "../types.js";

let _routeConfig: NuxtMuseaOptions["route"] = {};

export function _setRouteConfig(config: NuxtMuseaOptions["route"]): void {
  _routeConfig = config;
}

/**
 * Mock useRoute - returns a reactive route object.
 */
export function useRoute() {
  return reactive({
    path: _routeConfig?.path ?? "/",
    name: _routeConfig?.name ?? "index",
    params: _routeConfig?.params ?? {},
    query: _routeConfig?.query ?? {},
    hash: _routeConfig?.hash ?? "",
    fullPath: _routeConfig?.fullPath ?? _routeConfig?.path ?? "/",
    meta: _routeConfig?.meta ?? {},
    matched: [],
    redirectedFrom: undefined,
  });
}

/**
 * Mock useRouter - returns a router-like object with no-op navigation.
 */
export function useRouter() {
  return {
    push: async (_to: unknown) => {},
    replace: async (_to: unknown) => {},
    back: () => {},
    forward: () => {},
    go: (_delta: number) => {},
    resolve: (to: unknown) => ({
      href: typeof to === "string" ? to : "/",
      route: useRoute(),
    }),
    currentRoute: computed(() => useRoute()),
    addRoute: () => () => {},
    removeRoute: () => {},
    hasRoute: () => false,
    getRoutes: () => [],
    beforeEach: () => () => {},
    afterEach: () => () => {},
    onError: () => () => {},
    isReady: () => Promise.resolve(),
    options: {},
  };
}
