/**
 * Auto-imports virtual module.
 * Provides all Nuxt composable mocks via #imports alias.
 */

export { useRoute, useRouter } from "./mocks/composables.js";
export { useFetch, useAsyncData, useLazyFetch, useLazyAsyncData } from "./mocks/data.js";
export { navigateTo, abortNavigation, defineNuxtRouteMiddleware } from "./mocks/navigation.js";
export { useHead, useSeoMeta, useHeadSafe, useServerSeoMeta } from "./mocks/head.js";
export {
  useNuxtApp,
  useRuntimeConfig,
  useState,
  useRequestHeaders,
  useRequestEvent,
  useRequestURL,
  useCookie,
  clearNuxtState,
  defineNuxtPlugin,
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
