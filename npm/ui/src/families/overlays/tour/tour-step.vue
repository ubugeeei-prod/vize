<script setup lang="ts">
import { computed } from "vue";

import { tourContext } from "./tour-context.ts";
import type { TourStepSlotState } from "./tour-types.ts";

const { value } = defineProps<{
  /** Step value whose content this part renders. @default required */
  readonly value: string;
}>();

defineSlots<{
  /** Content rendered only while this step is current; the hidden wrapper stays mounted. */
  default(props: TourStepSlotState): unknown;
}>();

const context = tourContext.use();
const active = computed(() => context.step.value?.value === value);
const index = computed(() => context.indexOf(value));
const slotState = computed<TourStepSlotState>(() => ({
  index: index.value,
  total: context.total.value,
  value,
}));
</script>

<template>
  <div
    data-vize-ui="tour-step"
    part="step"
    :hidden="active ? undefined : true"
    :data-state="active ? 'active' : 'inactive'"
    :data-value="value"
    :data-index="index"
  >
    <slot v-if="active" v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
