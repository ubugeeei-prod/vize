<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { scrubberPreviewContext } from "./scrubber-preview-context.ts";
import { formatScrubberTime } from "./scrubber-preview-thumbnails.ts";
import type {
  ScrubberPreviewTimeExpose,
  ScrubberPreviewTimeSlotState,
} from "./scrubber-preview-types.ts";

const { format = undefined } = defineProps<{
  /**
   * Custom formatter. Defaults to `m:ss`, or `h:mm:ss` for media of an hour or longer.
   *
   * @default undefined
   */
  readonly format?: (seconds: number) => string;
}>();

defineSlots<{
  /** Custom time rendering. Receives the previewed time and its formatted text. */
  default(props: ScrubberPreviewTimeSlotState): unknown;
}>();

const context = scrubberPreviewContext.use();
const element = useTemplateRef<HTMLSpanElement>("element");
const text = computed(() => {
  const time = context.time.value;
  if (time === null) return "";
  return format?.(time) ?? formatScrubberTime(time, context.duration.value >= 3600);
});
const slotState = computed<ScrubberPreviewTimeSlotState>(() => ({
  text: text.value,
  time: context.time.value,
}));

type ScrubberPreviewTimeSetupExpose = Omit<
  ScrubberPreviewTimeExpose,
  "element" | "text" | "time"
> & {
  readonly element: typeof element;
  readonly text: ComputedRef<string>;
  readonly time: ComputedRef<number | null>;
};

const exposed = { element, text, time: context.time } satisfies ScrubberPreviewTimeSetupExpose;

defineExpose(exposed);
</script>

<template>
  <span
    ref="element"
    aria-hidden="true"
    data-vize-ui="scrubber-preview-time"
    part="time"
    :data-state="context.active.value ? 'active' : 'idle'"
  >
    <slot v-bind="slotState">{{ text }}</slot>
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
