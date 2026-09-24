<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { infiniteScrollContext } from "./infinite-scroll-context.ts";
import type {
  InfiniteScrollSlotState,
  InfiniteScrollState,
  InfiniteScrollStatusExpose,
} from "./infinite-scroll-types.ts";

defineSlots<{
  /**
   * Announcement text such as "Loading more results" or "All results loaded".
   * The polite live region is always rendered so changes are announced.
   */
  default(props: InfiniteScrollSlotState): unknown;
}>();

const context = infiniteScrollContext.use();
const element = useTemplateRef<HTMLDivElement>("element");

type InfiniteScrollStatusSetupExpose = Omit<
  InfiniteScrollStatusExpose,
  keyof InfiniteScrollSlotState | "element"
> & {
  readonly busy: ComputedRef<boolean>;
  readonly element: typeof element;
  readonly error: ComputedRef<unknown>;
  readonly hasMore: ComputedRef<boolean>;
  readonly state: ComputedRef<InfiniteScrollState>;
};

const exposed = {
  busy: computed(() => context.slotState.value.busy),
  element,
  error: computed(() => context.slotState.value.error),
  hasMore: computed(() => context.slotState.value.hasMore),
  state: context.state,
} satisfies InfiniteScrollStatusSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    role="status"
    aria-live="polite"
    aria-atomic="true"
    data-vize-ui="infinite-scroll-status"
    part="status"
    :data-state="context.state.value"
  >
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
