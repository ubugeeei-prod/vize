import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { ImageSlotState, ImageStatus } from "./image-types.ts";

/** Shared state and actions for the Image compound parts. */
export interface ImageContextValue {
  /** Current loading lifecycle state. */
  readonly status: ComputedRef<ImageStatus>;

  /** Source attached to the native image, or `undefined` while idle or failed. */
  readonly src: ComputedRef<string | undefined>;

  /** Zero-based index of the attached candidate, or `-1`. */
  readonly candidateIndex: ComputedRef<number>;

  /** Slot state shared by every part. */
  readonly slotState: ComputedRef<ImageSlotState>;

  /** Report a native `load` for the attached candidate. */
  readonly handleLoad: (nativeEvent: Event | null) => void;

  /** Report a native `error` for the attached candidate and advance the chain. */
  readonly handleError: (nativeEvent: Event | null) => void;
}

export const imageContext = createContext<ImageContextValue>("Image");
