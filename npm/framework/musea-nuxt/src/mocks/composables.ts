/**
 * Mock Nuxt routing composables.
 */

import { computed } from "vue";

import { getRouteState, resolveNavigationTarget, setRouteConfig } from "../context.js";
import type { NuxtMuseaNavigationTarget } from "../types.js";

export { setRouteConfig as _setRouteConfig };

/**
 * Mock useRoute - returns a reactive route object.
 */
export function useRoute() {
  return getRouteState();
}

/**
 * Mock useRouter - returns a router-like object with no-op navigation.
 */
export function useRouter() {
  const navigate = async (to: NuxtMuseaNavigationTarget) => {
    setRouteConfig(resolveNavigationTarget(to));
  };

  return {
    push: navigate,
    replace: navigate,
    back: () => {
      // no browser history in isolated previews
    },
    forward: () => {
      // no browser history in isolated previews
    },
    go: (_delta: number) => {
      // no browser history in isolated previews
    },
    resolve: (to: NuxtMuseaNavigationTarget) => ({
      href: resolveNavigationTarget(to).fullPath,
      route: resolveNavigationTarget(to),
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
