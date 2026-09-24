<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { infiniteScrollContext } from "./infinite-scroll-context.ts";
import type {
  InfiniteScrollLoadMoreExpose,
  InfiniteScrollSlotState,
  InfiniteScrollState,
} from "./infinite-scroll-types.ts";

const { hideWhenComplete = true } = defineProps<{
  /**
   * Apply the native `hidden` attribute once every item is loaded.
   *
   * @default true
   */
  readonly hideWhenComplete?: boolean;
}>();

const emit = defineEmits<{
  /** Fired before the request. Call `preventDefault()` to keep state unchanged. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Button label. Receives the loading state, e.g. to render "Retry" after an error. */
  default(props: InfiniteScrollSlotState): unknown;
}>();

const context = infiniteScrollContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const inactive = computed(() => {
  const state = context.state.value;
  return state === "complete" || state === "disabled" || state === "loading";
});
const hidden = computed(() => hideWhenComplete && context.state.value === "complete");

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) context.request("button");
}

type InfiniteScrollLoadMoreSetupExpose = Omit<
  InfiniteScrollLoadMoreExpose,
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
} satisfies InfiniteScrollLoadMoreSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled="inactive"
    :hidden="hidden ? true : undefined"
    :aria-controls="context.id.value"
    data-vize-ui="infinite-scroll-load-more"
    part="load-more"
    :data-state="context.state.value"
    @click="onClick"
  >
    <slot v-bind="context.slotState.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
