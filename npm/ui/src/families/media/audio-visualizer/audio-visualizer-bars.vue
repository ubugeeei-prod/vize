<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { audioVisualizerContext } from "./audio-visualizer-context.ts";
import type {
  AudioBarScale,
  AudioVisualizerBarsExpose,
  AudioVisualizerBarsSlotState,
} from "./audio-visualizer-types.ts";

const { count = 32, scale = "log" } = defineProps<{
  /**
   * Number of bars.
   *
   * @default 32
   */
  readonly count?: number;

  /**
   * Frequency grouping; `log` matches perceived pitch.
   *
   * @default "log"
   */
  readonly scale?: AudioBarScale;
}>();

defineSlots<{
  /** Custom bar rendering. Defaults to one `<span>` per bar with a height custom property. */
  default(props: AudioVisualizerBarsSlotState): unknown;
}>();

const context = audioVisualizerContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const bars = computed(() => context.bands(Math.max(0, Math.floor(count)), { scale }));
const slotState = computed<AudioVisualizerBarsSlotState>(() => ({
  bars: bars.value,
  state: context.state.value,
}));
const barStyles = computed(() =>
  bars.value.map((value, index) => ({
    id: `bar-${index}`,
    style: { "--vize-ui-audio-visualizer-bar": String(Math.round(value * 1000) / 1000) },
  })),
);

type AudioVisualizerBarsSetupExpose = Omit<AudioVisualizerBarsExpose, "bars" | "element"> & {
  readonly bars: ComputedRef<readonly number[]>;
  readonly element: typeof element;
};

const exposed = { bars, element } satisfies AudioVisualizerBarsSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    data-vize-ui="audio-visualizer-bars"
    part="bars"
    :data-count="barStyles.length"
    :data-state="context.state.value"
  >
    <slot v-bind="slotState">
      <span
        v-for="bar in barStyles"
        :key="bar.id"
        data-vize-ui="audio-visualizer-bar"
        part="bar"
        :style="bar.style"
      ></span>
    </slot>
  </div>
</template>

<style scoped>
/* Headless by design. Consumers size bars from --vize-ui-audio-visualizer-bar (0..1). */
</style>
