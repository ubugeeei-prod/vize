<script setup lang="ts" generic="T">
import { computed } from "vue";

import { chartContext } from "./chart-context.ts";
import type { ChartActivePoint } from "./chart-types.ts";

const { data, series = undefined } = defineProps<{
  /** Rows of the series the tooltip describes, so the slot receives a typed datum. @default required */
  readonly data: readonly T[];

  /**
   * Only show for this series name. `undefined` follows any active point.
   *
   * @default undefined
   */
  readonly series?: string;
}>();

defineSlots<{
  /** Tooltip content for the active point. */
  default?(props: {
    /** Active row. */
    readonly datum: T;
    /** Index of the active row. */
    readonly index: number;
    /** Active point geometry and label. */
    readonly point: ChartActivePoint;
  }): unknown;
}>();

const context = chartContext.use();
const point = computed(() => {
  const active = context.active.value;
  if (active === null || (series !== undefined && active.series !== series)) return null;
  return active;
});
const datum = computed(() => (point.value === null ? undefined : data[point.value.index]));
const style = computed(() =>
  point.value === null
    ? undefined
    : {
        "--vize-chart-tooltip-x": `${point.value.x + context.margin.value.left}px`,
        "--vize-chart-tooltip-y": `${point.value.y + context.margin.value.top}px`,
      },
);
</script>

<template>
  <div
    aria-hidden="true"
    :hidden="datum === undefined ? true : undefined"
    :style="style"
    data-vize-ui="chart-tooltip"
    part="tooltip"
    :data-state="datum === undefined ? 'closed' : 'open'"
    :data-series="point?.series"
  >
    <slot
      v-if="datum !== undefined && point !== null"
      :datum="datum"
      :index="point.index"
      :point="point"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Position the tooltip from its CSS variables. */
</style>
