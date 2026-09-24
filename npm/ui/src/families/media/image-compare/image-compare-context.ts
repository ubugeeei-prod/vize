import type { ComputedRef, Ref } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type {
  ImageCompareChangeSource,
  ImageCompareDirection,
  ImageCompareMessages,
  ImageCompareOrientation,
  ImageCompareSlotState,
  ImageCompareState,
} from "./image-compare-types.ts";

/** Shared state and actions for the ImageCompare compound parts. */
export interface ImageCompareContextValue {
  readonly handleId: ComputedRef<string>;
  readonly position: ComputedRef<number>;
  readonly orientation: ComputedRef<ImageCompareOrientation>;
  readonly dir: ComputedRef<ImageCompareDirection>;
  readonly disabled: ComputedRef<boolean>;
  readonly state: ComputedRef<ImageCompareState>;
  readonly step: ComputedRef<number>;
  readonly pageStep: ComputedRef<number>;
  readonly messages: ComputedRef<ImageCompareMessages>;
  readonly slotState: ComputedRef<ImageCompareSlotState>;
  readonly handleElement: Ref<HTMLElement | null>;
  readonly setPosition: (
    position: number,
    source: ImageCompareChangeSource,
    event: Event | null,
  ) => boolean;
}

export const imageCompareContext = createContext<ImageCompareContextValue>("ImageCompare");
