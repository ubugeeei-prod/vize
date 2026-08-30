<script setup lang="ts">
import { toRef, useTemplateRef } from "vue";

import { useLiveRegion } from "./live-region-runtime.ts";
import type { LiveRegionPoliteness } from "./live-region-types.ts";

const { politeness = "polite", atomic = true } = defineProps<{
  /**
   * Default announcement urgency.
   *
   * @default "polite"
   */
  readonly politeness?: LiveRegionPoliteness;

  /**
   * Whether assistive technology should present the whole region on each update.
   *
   * @default true
   */
  readonly atomic?: boolean;
}>();

defineSlots<{
  /** Optional visible contents rendered alongside the announcer text. */
  default(): unknown;
}>();

const region = useLiveRegion({ politeness: toRef(() => politeness) });
const element = useTemplateRef<HTMLDivElement>("element");

defineExpose({
  element,
  announce: region.announce,
  clear: region.clear,
  message: region.message,
});
</script>

<template>
  <div
    ref="element"
    data-vize-ui="live-region"
    :aria-live="region.politeness.value"
    :aria-atomic="atomic ? 'true' : 'false'"
    :role="region.politeness.value === 'assertive' ? 'alert' : 'status'"
  >
    <slot />
    {{ region.message.value }}
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
