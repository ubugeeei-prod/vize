import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  ScrubberPreviewCue,
  ScrubberPreviewDirection,
  ScrubberPreviewKind,
  ScrubberPreviewSlotState,
  ScrubberPreviewSprite,
} from "./scrubber-preview-types.ts";

/** Shared state and actions for the ScrubberPreview parts. */
export interface ScrubberPreviewContextValue {
  readonly duration: ComputedRef<number>;
  readonly time: ComputedRef<number | null>;
  readonly ratio: ComputedRef<number | null>;
  readonly active: ComputedRef<boolean>;
  readonly dir: ComputedRef<ScrubberPreviewDirection>;
  readonly disabled: ComputedRef<boolean>;
  readonly kind: ComputedRef<ScrubberPreviewKind>;
  readonly sprite: ComputedRef<ScrubberPreviewSprite | undefined>;
  readonly cues: ComputedRef<readonly ScrubberPreviewCue[]>;
  readonly videoSrc: ComputedRef<string | undefined>;
  readonly captureWidth: ComputedRef<number>;
  readonly captureInterval: ComputedRef<number>;
  readonly cacheSize: ComputedRef<number>;
  readonly slotState: ComputedRef<ScrubberPreviewSlotState>;
  /** Show a preview for a ratio `0..1` along the track, or hide it with `null`. */
  readonly previewRatio: (ratio: number | null) => void;
  /** Request a seek to the time at a ratio. */
  readonly seekRatio: (ratio: number, nativeEvent: Event) => void;
}

export const scrubberPreviewContext = createContext<ScrubberPreviewContextValue>("ScrubberPreview");
