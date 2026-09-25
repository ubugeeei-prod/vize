import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { AudioAnalyserState, AudioBarsOptions } from "./audio-visualizer-types.ts";

/** Shared state for AudioVisualizer parts. */
export interface AudioVisualizerContextValue {
  /** Lifecycle state. */
  readonly state: ComputedRef<AudioAnalyserState>;

  /** Group the latest frequency frame into bars; reactive per frame. */
  readonly bands: (count: number, options?: AudioBarsOptions) => readonly number[];
}

export const audioVisualizerContext = createContext<AudioVisualizerContextValue>("AudioVisualizer");
