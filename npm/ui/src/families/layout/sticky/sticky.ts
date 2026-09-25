/** Headless sticky/affix wrapper that pins with native `position: sticky` and reports stuck state. */
export { default as Sticky, default as Affix } from "./sticky.vue";
export { isStickyStuck, stickyRootMargin } from "./sticky-geometry.ts";
export type { StickyEdges } from "./sticky-geometry.ts";
export type { StickyExpose, StickySide, StickySlotState, StickyState } from "./sticky-types.ts";
