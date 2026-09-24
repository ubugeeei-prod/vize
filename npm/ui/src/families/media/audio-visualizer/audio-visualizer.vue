<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { audioVisualizerContext } from "./audio-visualizer-context.ts";
import { toWaveformPath } from "./audio-visualizer-math.ts";
import { useAudioAnalyser } from "./audio-visualizer-runtime.ts";
import type {
  AudioAnalyserPrecision,
  AudioAnalyserReducedMotion,
  AudioAnalyserSource,
  AudioAnalyserState,
  AudioBarsOptions,
  AudioVisualizerExpose,
  AudioVisualizerSlotState,
} from "./audio-visualizer-types.ts";

const {
  source = null,
  precision = "byte",
  fftSize = 2048,
  smoothingTimeConstant = 0.8,
  minDecibels = -100,
  maxDecibels = -30,
  frameRate = undefined,
  paused = false,
  reducedMotion = "throttle",
  connectToDestination = true,
  ariaLabel = undefined,
} = defineProps<{
  /**
   * Media element, stream (e.g. from `useUserMedia()`), or audio node to analyse.
   *
   * @default null
   */
  readonly source?: AudioAnalyserSource | null;

  /**
   * Sample precision, read once at setup: `byte` (`Uint8Array`) or `float` (`Float32Array`).
   *
   * @default "byte"
   */
  readonly precision?: AudioAnalyserPrecision;

  /**
   * FFT size, a power of two from 32 to 32768.
   *
   * @default 2048
   */
  readonly fftSize?: number;

  /**
   * Averaging constant between frames, `0..1`.
   *
   * @default 0.8
   */
  readonly smoothingTimeConstant?: number;

  /**
   * Lower bound of the decibel range used for normalization.
   *
   * @default -100
   */
  readonly minDecibels?: number;

  /**
   * Upper bound of the decibel range used for normalization.
   *
   * @default -30
   */
  readonly maxDecibels?: number;

  /**
   * Maximum frames per second; `undefined` reads every animation frame.
   *
   * @default undefined
   */
  readonly frameRate?: number;

  /**
   * Stop reading frames while `true`.
   *
   * @default false
   */
  readonly paused?: boolean;

  /**
   * Behavior under `prefers-reduced-motion: reduce`.
   *
   * @default "throttle"
   */
  readonly reducedMotion?: AudioAnalyserReducedMotion;

  /**
   * Keep media-element audio audible after analysis routing.
   *
   * @default true
   */
  readonly connectToDestination?: boolean;

  /**
   * Accessible name. Without one the visualization is decorative (`aria-hidden`).
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Render bars, a waveform, or a canvas from the latest frame. */
  default(props: AudioVisualizerSlotState<AudioAnalyserPrecision>): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const shared = {
  connectToDestination,
  fftSize: () => fftSize,
  frameRate: () => frameRate,
  maxDecibels: () => maxDecibels,
  minDecibels: () => minDecibels,
  paused: () => paused,
  reducedMotion: () => reducedMotion,
  smoothingTimeConstant: () => smoothingTimeConstant,
  source: () => source,
};
const analyser =
  precision === "float"
    ? useAudioAnalyser({ ...shared, precision: "float" })
    : useAudioAnalyser({ ...shared, precision: "byte" });
const state = computed<AudioAnalyserState>(() => analyser.state.value);
const level = computed(() => analyser.level.value);
const levelStyle = computed(() => ({
  "--vize-ui-audio-visualizer-level": String(Math.round(level.value * 1000) / 1000),
  "--vize-ui-audio-visualizer-peak": String(Math.round(analyser.peak.value * 1000) / 1000),
}));

function bands(count: number, options?: AudioBarsOptions): readonly number[] {
  // Read the frame ref so render effects re-run on every analysed frame.
  void analyser.frequency.value;
  return analyser.bands(count, options);
}

const slotState = computed<AudioVisualizerSlotState<AudioAnalyserPrecision>>(() => ({
  bands,
  frequency: analyser.frequency.value,
  level: level.value,
  peak: analyser.peak.value,
  resume: analyser.resume,
  state: state.value,
  waveform: analyser.waveform.value,
  waveformPath: (width, height) => toWaveformPath(analyser.waveform.value, width, height),
}));

audioVisualizerContext.provide({ bands, state });

type AudioVisualizerSetupExpose = Omit<AudioVisualizerExpose, "element" | "level" | "state"> & {
  readonly element: typeof element;
  readonly level: ComputedRef<number>;
  readonly state: ComputedRef<AudioAnalyserState>;
};

const exposed = {
  element,
  level,
  resume: analyser.resume,
  state,
} satisfies AudioVisualizerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <div
    ref="element"
    :role="ariaLabel === undefined ? undefined : 'img'"
    :aria-label="ariaLabel"
    :aria-hidden="ariaLabel === undefined ? 'true' : undefined"
    data-vize-ui="audio-visualizer-root"
    part="root"
    :data-state="state"
    :style="levelStyle"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
