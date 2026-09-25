<script setup lang="ts">
import { chartContext } from "./chart-context.ts";
import type { ChartSeriesInfo } from "./chart-types.ts";

const { toggleable = true, ariaLabel = "Legend" } = defineProps<{
  /**
   * Render each entry as a toggle button that shows or hides its series.
   *
   * @default true
   */
  readonly toggleable?: boolean;

  /**
   * Accessible name of the legend list.
   *
   * @default "Legend"
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Entry content, for example a swatch and the series name. */
  default?(props: {
    /** Registered series. */
    readonly series: ChartSeriesInfo;
  }): unknown;
}>();

const context = chartContext.use();

function toggle(name: string): void {
  context.toggleSeries(name);
}
</script>

<template>
  <ul :aria-label="ariaLabel" data-vize-ui="chart-legend" part="legend">
    <li
      v-for="entry in context.series.value"
      :key="entry.name"
      data-vize-ui="chart-legend-item"
      part="item"
      :data-series="entry.name"
      :data-hidden="entry.hidden ? 'true' : undefined"
    >
      <button
        v-if="toggleable"
        type="button"
        :aria-pressed="entry.hidden ? 'false' : 'true'"
        data-vize-ui="chart-legend-toggle"
        @click="() => toggle(entry.name)"
      >
        <slot :series="entry">{{ entry.name }}</slot>
      </button>
      <slot v-else :series="entry">{{ entry.name }}</slot>
    </li>
  </ul>
</template>

<style scoped>
/* Headless by design. Swatches and layout remain consumer-owned. */
</style>
