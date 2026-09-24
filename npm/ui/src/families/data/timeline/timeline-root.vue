<script setup lang="ts">
import { computed, onMounted, shallowReactive, shallowRef, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { createCollectionRegistry } from "../../foundations/collection/collection.ts";
import { timelineContext } from "./timeline-context.ts";
import type { TimelineContextValue } from "./timeline-context.ts";
import type {
  TimelineItemStatus,
  TimelineOrientation,
  TimelineRootExpose,
  TimelineSlotState,
} from "./timeline-types.ts";

const {
  value = null,
  orientation = "vertical",
  reversed = false,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
} = defineProps<{
  /**
   * Value of the current item. Earlier items become `complete`, later ones `upcoming`.
   *
   * @default null
   */
  readonly value?: string | null;

  /**
   * Layout axis exposed to styles.
   *
   * @default "vertical"
   */
  readonly orientation?: TimelineOrientation;

  /**
   * List items newest first with the native `reversed` ordered-list semantics.
   *
   * @default false
   */
  readonly reversed?: boolean;

  /**
   * Accessible name for the list.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the list.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;
}>();

defineSlots<{
  /** TimelineItem children. Receives the current value, axis, and item count. */
  default(props: TimelineSlotState): unknown;
}>();

const element = useTemplateRef<HTMLOListElement>("element");
const registry = createCollectionRegistry<string, string>();
const itemValues = shallowReactive(new Map<string, string | null>());
const valueState = computed(() => value);
const orientationState = computed(() => orientation);
const keys = computed(() => registry.items.value.map((item) => item.key));
const values = computed(() =>
  keys.value.flatMap((key) => {
    const itemValue = itemValues.get(key);
    return itemValue === null || itemValue === undefined ? [] : [itemValue];
  }),
);
const currentIndex = computed(() =>
  valueState.value === null
    ? -1
    : keys.value.findIndex((key) => itemValues.get(key) === valueState.value),
);
const slotState = computed<TimelineSlotState>(() => ({
  count: keys.value.length,
  orientation: orientationState.value,
  reversed,
  value: valueState.value,
}));

// Items render in document order, so before mount (and on the server) an item rendered
// while the current value is still unregistered must precede it.
const mounted = shallowRef(false);
onMounted(() => {
  mounted.value = true;
});

function readCurrentIndex(): number {
  return currentIndex.value;
}

function getStatus(index: number, itemValue: string | null): TimelineItemStatus | null {
  const current = readCurrentIndex();
  if (index < 0 || valueState.value === null) return null;
  if (current < 0) {
    if (itemValue === valueState.value) return "current";
    return mounted.value ? null : "complete";
  }
  if (itemValue !== null && itemValue === valueState.value) return "current";
  return index < current ? "complete" : index > current ? "upcoming" : "current";
}

timelineContext.provide({
  getCount: () => keys.value.length,
  getIndex: (key) => keys.value.indexOf(key),
  getStatus,
  orientation: orientationState,
  registerItem: (input) => {
    const registration = registry.register({
      key: input.key,
      value: input.key,
      element: input.element,
    });
    return {
      ...registration,
      unregister: () => {
        itemValues.delete(input.key);
        return registration.unregister();
      },
    };
  },
  setValue: (key, itemValue) => {
    itemValues.set(key, itemValue);
  },
  value: valueState,
} satisfies TimelineContextValue);

type TimelineRootSetupExpose = Omit<TimelineRootExpose, "element" | "value" | "values"> & {
  readonly element: typeof element;
  readonly value: ComputedRef<string | null>;
  readonly values: ComputedRef<readonly string[]>;
};

const exposed = {
  element,
  value: valueState,
  values,
} satisfies TimelineRootSetupExpose;

defineExpose(exposed);
</script>

<template>
  <ol
    ref="element"
    :reversed="reversed ? true : undefined"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    data-vize-ui="timeline"
    part="root"
    :data-orientation="orientationState"
    :data-reversed="reversed ? 'true' : undefined"
  >
    <slot v-bind="slotState" />
  </ol>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
