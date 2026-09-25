/** Headless Web Audio visualizer: a typed analyser composable, pure bar/waveform helpers, and SFCs. */
export { default as AudioVisualizer } from "./audio-visualizer.vue";
export { default as AudioVisualizerBars } from "./audio-visualizer-bars.vue";
export {
  barEdges,
  computeLevel,
  computePeak,
  toBars,
  toWaveformPath,
} from "./audio-visualizer-math.ts";
export { useAudioAnalyser } from "./audio-visualizer-runtime.ts";
export type {
  AudioAnalyser,
  AudioAnalyserData,
  AudioAnalyserPrecision,
  AudioAnalyserReducedMotion,
  AudioAnalyserSource,
  AudioAnalyserState,
  AudioAnalyserStreamLike,
  AudioBarScale,
  AudioBarsOptions,
  AudioVisualizerBarsExpose,
  AudioVisualizerBarsSlotState,
  AudioVisualizerExpose,
  AudioVisualizerSlotState,
  UseAudioAnalyserOptions,
} from "./audio-visualizer-types.ts";
