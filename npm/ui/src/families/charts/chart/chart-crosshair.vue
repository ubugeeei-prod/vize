<script setup lang="ts">
import { computed } from "vue";

import { chartContext } from "./chart-context.ts";

const { axis = "x", series = undefined } = defineProps<{
  /**
   * Which guide lines to draw through the active point.
   *
   * @default "x"
   */
  readonly axis?: "both" | "x" | "y";

  /**
   * Only follow this series name. `undefined` follows any active point.
   *
   * @default undefined
   */
  readonly series?: string;
}>();

const context = chartContext.use();
const point = computed(() => {
  const active = context.active.value;
  if (active === null || (series !== undefined && active.series !== series)) return null;
  return active;
});
</script>

<template>
  <g
    aria-hidden="true"
    data-vize-ui="chart-crosshair"
    part="crosshair"
    :data-state="point === null ? 'hidden' : 'visible'"
  >
    <template v-if="point !== null">
      <line
        v-if="axis !== 'y'"
        :x1="point.x"
        :x2="point.x"
        :y1="0"
        :y2="context.innerHeight.value"
        data-vize-ui="chart-crosshair-x"
      />
      <line
        v-if="axis !== 'x'"
        :x1="0"
        :x2="context.innerWidth.value"
        :y1="point.y"
        :y2="point.y"
        data-vize-ui="chart-crosshair-y"
      />
    </template>
  </g>
</template>

<style scoped>
/* Headless by design. Stroke styling remains consumer-owned. */
</style>
