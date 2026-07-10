/**
 * Auto-imports virtual module.
 * Provides all Nuxt composable mocks via #imports alias.
 */

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

// Re-export Vue core composables that Nuxt auto-imports
export {
  ref,
  reactive,
  computed,
  watch,
  watchEffect,
  onMounted,
  onUnmounted,
  onBeforeMount,
  onBeforeUnmount,
  onUpdated,
  onBeforeUpdate,
  onActivated,
  onDeactivated,
  onErrorCaptured,
  provide,
  inject,
  nextTick,
  defineComponent,
  defineAsyncComponent,
  toRef,
  toRefs,
  toRaw,
  unref,
  isRef,
  isReactive,
  isReadonly,
  isProxy,
  shallowRef,
  shallowReactive,
  shallowReadonly,
  triggerRef,
  customRef,
  markRaw,
  effectScope,
  getCurrentScope,
  onScopeDispose,
  readonly,
  toValue,
  useAttrs,
  useSlots,
  h,
} from "vue";
