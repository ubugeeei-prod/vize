<script setup lang="ts" generic="Data = unknown">
import { computed, onMounted, onUnmounted, provide, watch } from "vue";
import type { InjectionKey } from "vue";

import { toastProviderContext, toastStoreKey } from "./toast-context.ts";
import { createToastStore } from "./toast-store.ts";
import type { ToastProviderExpose, ToastStore, ToastSwipeDirection } from "./toast-types.ts";

const {
  store: externalStore = undefined,
  duration = undefined,
  limit = undefined,
  label = "Notifications",
  hotkey = ["F8"],
  swipeDirection = "right",
  swipeThreshold = 50,
  pauseOnPageHidden = true,
} = defineProps<{
  /**
   * Externally created store, for example a module-level store shared with
   * code outside components. `undefined` creates a request-local store.
   *
   * @default undefined
   */
  readonly store?: ToastStore<Data>;

  /**
   * Default auto-dismiss duration in milliseconds. `undefined` keeps the store default (5000).
   *
   * @default undefined
   */
  readonly duration?: number;

  /**
   * Maximum number of visible toasts. `undefined` keeps the store default (3).
   *
   * @default undefined
   */
  readonly limit?: number;

  /**
   * Accessible name of the notification region, suffixed with the hotkey.
   *
   * @default "Notifications"
   */
  readonly label?: string;

  /**
   * Keys that move focus to the region. Modifier tokens are `altKey`,
   * `ctrlKey`, `metaKey`, and `shiftKey`; other tokens match `code` or `key`.
   *
   * @default ["F8"]
   */
  readonly hotkey?: readonly string[];

  /**
   * Direction a toast must be swiped to dismiss it.
   *
   * @default "right"
   */
  readonly swipeDirection?: ToastSwipeDirection;

  /**
   * Pixels a swipe must travel before release dismisses the toast.
   *
   * @default 50
   */
  readonly swipeThreshold?: number;

  /**
   * Pause timers while `document.visibilityState` is `hidden`.
   *
   * @default true
   */
  readonly pauseOnPageHidden?: boolean;
}>();

defineSlots<{
  /** Application subtree, usually including a ToastViewport. */
  default?(props: { readonly store: ToastStore<Data> }): unknown;
}>();

const store =
  externalStore ??
  createToastStore<Data>({
    ...(duration === undefined ? {} : { duration }),
    ...(limit === undefined ? {} : { limit }),
  });
const storeKey: InjectionKey<ToastStore<Data>> = toastStoreKey;
provide(storeKey, store);

watch(
  () => [duration, limit] as const,
  ([nextDuration, nextLimit]) => {
    store.configure({
      ...(nextDuration === undefined ? {} : { duration: nextDuration }),
      ...(nextLimit === undefined ? {} : { limit: nextLimit }),
    });
  },
  { immediate: externalStore !== undefined },
);

toastProviderContext.provide({
  label: computed(() => label),
  hotkey: computed(() => hotkey),
  swipeDirection: computed(() => swipeDirection),
  swipeThreshold: computed(() => (Number.isFinite(swipeThreshold) ? swipeThreshold : 50)),
});

let ownerDocument: Document | null = null;

function syncVisibility(): void {
  if (!ownerDocument) return;
  if (pauseOnPageHidden && ownerDocument.visibilityState === "hidden") store.pause("hidden");
  else store.resume("hidden");
}

onMounted(() => {
  ownerDocument = globalThis.document;
  ownerDocument.addEventListener("visibilitychange", syncVisibility);
  syncVisibility();
  store.start();
});

watch(() => pauseOnPageHidden, syncVisibility);

onUnmounted(() => {
  ownerDocument?.removeEventListener("visibilitychange", syncVisibility);
  ownerDocument = null;
  store.resume("hidden");
  store.stop();
});

const exposed = { store } satisfies ToastProviderExpose<Data>;

defineExpose(exposed);
</script>

<template>
  <div data-vize-ui="toast-provider" part="provider">
    <slot :store />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
