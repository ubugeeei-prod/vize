/** Headless before/after image comparison with a WAI-ARIA slider handle. */
export { default as ImageCompareAfter } from "./image-compare-after.vue";
export { default as ImageCompareBefore } from "./image-compare-before.vue";
export { default as ImageCompareHandle } from "./image-compare-handle.vue";
export { default as ImageCompareLabel } from "./image-compare-label.vue";
export { default as ImageCompare, default as ImageCompareRoot } from "./image-compare-root.vue";
export {
  imageComparePositionForKey,
  imageComparePositionFromPoint,
  normalizeImageComparePosition,
} from "./image-compare-value.ts";
export type {
  ImageCompareAxis,
  ImageCompareKeyOptions,
  ImageCompareRect,
} from "./image-compare-value.ts";
export type {
  ImageCompareChangeSource,
  ImageCompareDirection,
  ImageCompareHandleExpose,
  ImageCompareMessages,
  ImageCompareMode,
  ImageCompareOrientation,
  ImageComparePartExpose,
  ImageCompareRootExpose,
  ImageCompareSide,
  ImageCompareSlotState,
  ImageCompareState,
} from "./image-compare-types.ts";
