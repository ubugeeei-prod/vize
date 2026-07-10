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
export {
  configureNuxtMuseaMocks,
  resetNuxtMuseaMocks,
  getRouteState,
  getRuntimeConfigState,
  getAppConfigState,
} from "./context.js";
export { installNuxtMuseaMocks, createNuxtMuseaPreviewSetup } from "./app.js";

// Re-export mock composables for direct use
export { useRoute, useRouter } from "./mocks/composables.js";
export {
  useFetch,
  useAsyncData,
  useLazyFetch,
  useLazyAsyncData,
  useNuxtData,
  refreshNuxtData,
  clearNuxtData,
  useRequestFetch,
} from "./mocks/data.js";
export {
  navigateTo,
  abortNavigation,
  defineNuxtRouteMiddleware,
  definePageMeta,
  setPageLayout,
  prefetchComponents,
  preloadComponents,
  preloadRouteComponents,
} from "./mocks/navigation.js";
export { useHead, useSeoMeta, useHeadSafe, useServerSeoMeta } from "./mocks/head.js";
export { createError, showError, clearError, useError } from "./mocks/error.js";
export {
  useNuxtApp,
  useRuntimeConfig,
  useAppConfig,
  updateAppConfig,
  useState,
  useRequestHeaders,
  useRequestEvent,
  useRequestURL,
  useCookie,
  clearNuxtState,
  defineNuxtPlugin,
  defineNuxtComponent,
  onNuxtReady,
  callOnce,
  useId,
  reloadNuxtApp,
} from "./mocks/runtime.js";
export {
  NuxtLink,
  NuxtPage,
  ClientOnly,
  NuxtLayout,
  NuxtLoadingIndicator,
  NuxtErrorBoundary,
  NuxtRouteAnnouncer,
  NuxtWelcome,
  NuxtIsland,
  NuxtClientFallback,
  NuxtImg,
  NuxtPicture,
} from "./mocks/components.js";
