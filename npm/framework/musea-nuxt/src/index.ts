/**
 * @vizejs/musea-nuxt
 *
 * Nuxt mock layer for Musea - enables Nuxt component isolation in galleries.
 *
 * @example
 * ```ts
 * import { defineConfig } from 'vite'
 * import { musea } from '@vizejs/vite-plugin-musea'
 * import { nuxtMusea } from '@vizejs/musea-nuxt'
 *
 * export default defineConfig({
 *   plugins: [
 *     musea(),
 *     nuxtMusea({
 *       route: { path: '/', params: {} },
 *       runtimeConfig: { public: { apiBase: '/api' } },
 *       fetchMocks: { '/api/users': [{ id: 1, name: 'Alice' }] },
 *     }),
 *   ],
 * })
 * ```
 */

import type { Plugin } from "vite";
import { createNuxtMuseaPlugin } from "./plugin.js";
import type { NuxtMuseaOptions } from "./types.js";

/**
 * Create Nuxt mock Vite plugin for Musea.
 */
export function nuxtMusea(options: NuxtMuseaOptions = {}): Plugin {
  return createNuxtMuseaPlugin(options);
}

export type { NuxtMuseaOptions } from "./types.js";

// Re-export mock composables for direct use
export { useRoute, useRouter } from "./mocks/composables.js";
export { useFetch, useAsyncData, useLazyFetch, useLazyAsyncData } from "./mocks/data.js";
export { navigateTo, abortNavigation } from "./mocks/navigation.js";
export { useHead, useSeoMeta } from "./mocks/head.js";
export { useNuxtApp, useRuntimeConfig, useState, useCookie } from "./mocks/runtime.js";
export {
  NuxtLink,
  NuxtPage,
  ClientOnly,
  NuxtLayout,
  NuxtLoadingIndicator,
  NuxtErrorBoundary,
} from "./mocks/components.js";
