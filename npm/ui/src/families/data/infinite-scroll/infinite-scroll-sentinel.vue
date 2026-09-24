<script setup lang="ts">
import { onMounted, shallowRef, useTemplateRef, watch } from "vue";
import type { ShallowRef } from "vue";

import { useVisibilityObserver } from "../../interaction/measure/measure-runtime.ts";
import { infiniteScrollContext } from "./infinite-scroll-context.ts";
import type {
  InfiniteScrollSentinelExpose,
  InfiniteScrollSlotState,
} from "./infinite-scroll-types.ts";

defineSlots<{
  /** Optional sentinel content such as a spinner. Receives the loading state. */
  default(props: InfiniteScrollSlotState): unknown;
}>();

const context = infiniteScrollContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const intersecting = shallowRef(false);

// The intersection root and margin are read when observation starts on mount.
const observer = useVisibilityObserver({
  get root() {
    return context.scrollRoot.value === "self" ? context.element.value : null;
  },
  get rootMargin() {
    return context.rootMargin.value;
  },
  onVisibilityChange(entries) {
    const latest = entries.at(-1);
    if (latest === undefined) return;
    intersecting.value = latest.isIntersecting;
    if (latest.isIntersecting) context.request("sentinel");
  },
});

onMounted(() => {
  if (element.value !== null) observer.observe(element.value);
});

// Re-observing delivers a fresh initial entry, so a sentinel that stayed visible
// through a load (a short page) requests the next page once the root is idle again.
watch(context.refreshToken, () => {
  if (element.value === null) return;
  observer.unobserve(element.value);
  observer.observe(element.value);
});

type InfiniteScrollSentinelSetupExpose = Omit<
  InfiniteScrollSentinelExpose,
  "element" | "intersecting"
> & {
  readonly element: typeof element;
  readonly intersecting: Readonly<ShallowRef<boolean>>;
};

const exposed = { element, intersecting } satisfies InfiniteScrollSentinelSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    aria-hidden="true"
    data-vize-ui="infinite-scroll-sentinel"
    part="sentinel"
    :data-state="context.state.value"
  >
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
