import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { RangeSliderBounds } from "./range-slider-state.ts";
import type {
  RangeSliderChangeSource,
  RangeSliderDirection,
  RangeSliderOrientation,
  RangeSliderValue,
  RangeSliderValueText,
} from "./range-slider-types.ts";

/** Shared state and actions for RangeSlider parts. */
export interface RangeSliderContextValue {
  readonly values: ComputedRef<RangeSliderValue>;
  readonly bounds: ComputedRef<RangeSliderBounds>;
  readonly orientation: ComputedRef<RangeSliderOrientation>;
  readonly direction: ComputedRef<RangeSliderDirection>;
  readonly disabled: ComputedRef<boolean>;
  readonly activeThumb: ShallowRef<number | null>;
  readonly getValueText: ComputedRef<RangeSliderValueText | undefined>;
  readonly ariaLabelledby: ComputedRef<string | undefined>;
  readonly ariaDescribedby: ComputedRef<string | undefined>;
  readonly ariaErrormessage: ComputedRef<string | undefined>;
  readonly ariaInvalid: ComputedRef<"grammar" | "spelling" | "true" | undefined>;
  readonly registerThumb: (index: number, element: HTMLElement | null) => void;
  readonly focusThumb: (index: number, options?: FocusOptions) => void;
  readonly setThumb: (index: number, value: number) => boolean;
  readonly commit: (index: number, source: RangeSliderChangeSource) => void;
  readonly startDrag: (event: PointerEvent, track: HTMLElement) => void;
}

/** Typed context shared by RangeSlider, its track, range, and thumbs. */
export const rangeSliderContext = createContext<RangeSliderContextValue>("RangeSlider");
