/** Stack several sticky headers/toolbars so each sticks below the previous ones. */
export { default as StickyStack } from "./sticky-stack.vue";
export { default as StickyStackItem } from "./sticky-stack-item.vue";
export { computeStickyOffsets, sortByDocumentOrder } from "./sticky-stack-layout.ts";
export type { StickyOffsets } from "./sticky-stack-layout.ts";
export type { StickyStackItemSlotState, StickyStackSlotState } from "./sticky-stack-types.ts";
