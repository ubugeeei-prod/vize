/** Headless gallery lightbox on the Dialog family: keyboard, swipe, thumbnails, and preloading. */
export { default as LightboxClose } from "./lightbox-close.vue";
export { default as LightboxContent } from "./lightbox-content.vue";
export { default as LightboxCounter } from "./lightbox-counter.vue";
export { default as LightboxImage } from "./lightbox-image.vue";
export { default as LightboxItem } from "./lightbox-item.vue";
export { default as LightboxNext } from "./lightbox-next.vue";
export { default as LightboxPrevious } from "./lightbox-previous.vue";
export { default as Lightbox, default as LightboxRoot } from "./lightbox-root.vue";
export { default as LightboxThumbnail } from "./lightbox-thumbnail.vue";
export { default as LightboxThumbnails } from "./lightbox-thumbnails.vue";
export { default as LightboxTrigger } from "./lightbox-trigger.vue";
export {
  classifyLightboxSwipe,
  lightboxPreloadIndexes,
  resolveLightboxIndex,
  resolveLightboxMessages,
} from "./lightbox-state.ts";
export type { LightboxSwipe, LightboxSwipeOptions } from "./lightbox-state.ts";
export { defaultLightboxMessages } from "./lightbox-types.ts";
export type {
  LightboxButtonExpose,
  LightboxChangeReason,
  LightboxContentExpose,
  LightboxCounterSlotState,
  LightboxDirection,
  LightboxElementExpose,
  LightboxIndexSlotState,
  LightboxItemExpose,
  LightboxMessageOverrides,
  LightboxMessages,
  LightboxPartSlotState,
  LightboxRootExpose,
  LightboxSlotState,
  LightboxState,
} from "./lightbox-types.ts";
