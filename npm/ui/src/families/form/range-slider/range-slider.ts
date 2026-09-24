/** Multi-thumb APG slider for value ranges with pointer dragging, keyboard, and form association. */
export { default as RangeSlider } from "./range-slider.vue";
/** Pointer surface that moves the closest thumb and drags it. */
export { default as RangeSliderTrack } from "./range-slider-track.vue";
/** Filled segment between the first and last thumbs. */
export { default as RangeSliderRange } from "./range-slider-range.vue";
/** One focusable `role="slider"` thumb bound to a value index. */
export { default as RangeSliderThumb } from "./range-slider-thumb.vue";
export {
  closestRangeSliderThumb,
  getRangeSliderThumbBounds,
  normalizeRangeSliderBounds,
  normalizeRangeSliderValue,
  setRangeSliderThumb,
} from "./range-slider-state.ts";
export type { RangeSliderBounds, RangeSliderBoundsOptions } from "./range-slider-state.ts";
export type {
  RangeSliderAriaInvalid,
  RangeSliderChangeSource,
  RangeSliderDirection,
  RangeSliderEmits,
  RangeSliderExpose,
  RangeSliderOrientation,
  RangeSliderProps,
  RangeSliderSlotState,
  RangeSliderSlots,
  RangeSliderState,
  RangeSliderStyle,
  RangeSliderThumbProps,
  RangeSliderThumbSlotState,
  RangeSliderThumbStyle,
  RangeSliderValue,
  RangeSliderValueText,
} from "./range-slider-types.ts";
