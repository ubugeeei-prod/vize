/** Headless WAI-ARIA carousel on native scroll snapping, with autoplay, drag, and slide picker. */
export { default as CarouselAutoplayToggle } from "./carousel-autoplay-toggle.vue";
export { default as CarouselIndicator } from "./carousel-indicator.vue";
export { default as CarouselIndicatorGroup } from "./carousel-indicator-group.vue";
export { default as CarouselNext } from "./carousel-next.vue";
export { default as CarouselPrevious } from "./carousel-previous.vue";
export { default as Carousel, default as CarouselRoot } from "./carousel-root.vue";
export { default as CarouselSlide } from "./carousel-slide.vue";
export { default as CarouselViewport } from "./carousel-viewport.vue";
export {
  nearestSlideIndex,
  resolveSlideIndex,
  scrollEdges,
  scrollPositionForSlide,
  slideIndexAfterDrag,
  slideStartOffset,
} from "./carousel-geometry.ts";
export type {
  CarouselAxis,
  CarouselRect,
  CarouselScrollEdges,
  CarouselScrollMetrics,
} from "./carousel-geometry.ts";
export type {
  CarouselAutoplayState,
  CarouselButtonExpose,
  CarouselChangeReason,
  CarouselDirection,
  CarouselFocusBehavior,
  CarouselIndicatorGroupExpose,
  CarouselIndicatorSlotState,
  CarouselOrientation,
  CarouselPauseReason,
  CarouselRootExpose,
  CarouselSlideExpose,
  CarouselSlideSlotState,
  CarouselSlideState,
  CarouselSlotState,
  CarouselViewportExpose,
} from "./carousel-types.ts";
