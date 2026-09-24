/** Headless, reduced-motion aware marquee driven by measured CSS custom properties. */
export { default as MarqueeContent } from "./marquee-content.vue";
export { default as MarqueePauseButton } from "./marquee-pause-button.vue";
export { default as Marquee, default as MarqueeRoot } from "./marquee-root.vue";
export {
  MARQUEE_MAX_COPIES,
  MARQUEE_MIN_COPIES,
  marqueeCopies,
  marqueeDuration,
  marqueeOrientation,
  marqueeStyle,
} from "./marquee-geometry.ts";
export { defaultMarqueeMessages, resolveMarqueeMessages } from "./marquee-types.ts";
export type {
  MarqueeContentExpose,
  MarqueeDirection,
  MarqueeMeasurement,
  MarqueeMessageOverrides,
  MarqueeMessages,
  MarqueeOrientation,
  MarqueePauseButtonExpose,
  MarqueePauseReason,
  MarqueeRootExpose,
  MarqueeSlotState,
  MarqueeState,
} from "./marquee-types.ts";
