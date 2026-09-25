<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { lightboxContext } from "./lightbox-context.ts";
import type { LightboxElementExpose, LightboxPartSlotState } from "./lightbox-types.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible tablist name. Defaults to the `thumbnails` message.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** LightboxThumbnail children. */
  default(props: LightboxPartSlotState): unknown;
}>();

const context = lightboxContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const label = computed<string>(() => ariaLabel ?? context.messages.value.thumbnails);

const exposed = { element } satisfies Omit<LightboxElementExpose, "element"> & {
  readonly element: typeof element;
};

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    role="tablist"
    aria-orientation="horizontal"
    :aria-label="label"
    data-vize-ui="lightbox-thumbnails"
    part="thumbnails"
  >
    <slot v-bind="context.slotState.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
