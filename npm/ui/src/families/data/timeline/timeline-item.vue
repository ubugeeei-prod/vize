<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { timelineContext, timelineItemContext } from "./timeline-context.ts";
import type {
  TimelineItemExpose,
  TimelineItemSlotState,
  TimelineItemStatus,
} from "./timeline-types.ts";

const { value = null, status = undefined } = defineProps<{
  /**
   * Item value compared with the root `value` to derive progress.
   *
   * @default null
   */
  readonly value?: string | null;

  /**
   * Explicit progress status that overrides the derived one.
   *
   * @default undefined
   */
  readonly status?: TimelineItemStatus;
}>();

defineSlots<{
  /** Item content. Receives the index, progress status, and whether this is the last item. */
  default(props: TimelineItemSlotState): unknown;
}>();

const context = timelineContext.use();
const element = useTemplateRef<HTMLLIElement>("element");
const key = useDeterministicId({ hint: "timeline-item" });
let registration: CollectionRegistration<string> | null = null;
watch(
  key,
  (next) => {
    registration?.unregister();
    registration = context.registerItem({ element, key: next });
  },
  { flush: "sync", immediate: true },
);
watch([key, () => value], ([next, itemValue]) => context.setValue(next, itemValue), {
  flush: "sync",
  immediate: true,
});
onScopeDispose(() => {
  registration?.unregister();
  registration = null;
});

const index = computed(() => context.getIndex(key.value));
const valueState = computed(() => value);
const itemStatus = computed<TimelineItemStatus | null>(
  () => status ?? context.getStatus(index.value, value),
);
const last = computed(() => index.value === context.getCount() - 1);
const slotState = computed<TimelineItemSlotState>(() => ({
  index: index.value,
  last: last.value,
  status: itemStatus.value,
  value,
}));

timelineItemContext.provide({ index, last, status: itemStatus, value: valueState });

type TimelineItemSetupExpose = {
  readonly element: typeof element;
  readonly index: ComputedRef<number>;
  readonly last: ComputedRef<boolean>;
  readonly status: ComputedRef<TimelineItemStatus | null>;
  readonly value: ComputedRef<string | null>;
} & Omit<TimelineItemExpose, "element" | "index" | "last" | "status" | "value">;

const exposed = {
  element,
  index,
  last,
  status: itemStatus,
  value: valueState,
} satisfies TimelineItemSetupExpose;

defineExpose(exposed);
</script>

<template>
  <li
    ref="element"
    :aria-current="itemStatus === 'current' ? 'step' : undefined"
    data-vize-ui="timeline-item"
    part="item"
    :data-state="itemStatus ?? undefined"
    :data-index="index"
    :data-last="last ? 'true' : undefined"
    :data-value="value ?? undefined"
    :data-orientation="context.orientation.value"
  >
    <slot v-bind="slotState" />
  </li>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
