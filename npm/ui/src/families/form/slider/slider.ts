/** Accessible, unstyled native range slider with controlled and uncontrolled state. */
export { default as Slider } from "./slider.vue";
export {
  SLIDER_DEFAULT_MAX,
  SLIDER_DEFAULT_MIN,
  SLIDER_DEFAULT_STEP,
  getSliderState,
} from "./slider-state.ts";
export type { SliderStateOptions } from "./slider-state.ts";
export type {
  SliderAriaInvalid,
  SliderDirection,
  SliderEmits,
  SliderExpose,
  SliderOrientation,
  SliderProps,
  SliderSlotState,
  SliderSlots,
  SliderState,
  SliderStep,
  SliderStyle,
} from "./slider-types.ts";
