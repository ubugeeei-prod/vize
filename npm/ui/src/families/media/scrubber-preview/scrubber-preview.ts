/** Headless seek-bar preview: sprite, WebVTT, or captured video thumbnails with hover time. */
export {
  default as ScrubberPreview,
  default as ScrubberPreviewRoot,
} from "./scrubber-preview-root.vue";
export { default as ScrubberPreviewThumbnail } from "./scrubber-preview-thumbnail.vue";
export { default as ScrubberPreviewTime } from "./scrubber-preview-time.vue";
export { default as ScrubberPreviewTrack } from "./scrubber-preview-track.vue";
export { captureVideoFrame, ScrubberPreviewError } from "./scrubber-preview-capture.ts";
export {
  createFrameCache,
  findThumbnailCue,
  formatScrubberTime,
  parseThumbnailVtt,
  quantizeTime,
  ratioFromPointer,
  spriteFrame,
} from "./scrubber-preview-thumbnails.ts";
export type { FrameCache } from "./scrubber-preview-thumbnails.ts";
export type {
  CaptureVideoFrameOptions,
  ScrubberPreviewCue,
  ScrubberPreviewDirection,
  ScrubberPreviewErrorCode,
  ScrubberPreviewFrame,
  ScrubberPreviewKind,
  ScrubberPreviewRegion,
  ScrubberPreviewRootExpose,
  ScrubberPreviewSlotState,
  ScrubberPreviewSprite,
  ScrubberPreviewStatus,
  ScrubberPreviewThumbnailExpose,
  ScrubberPreviewThumbnailSlotState,
  ScrubberPreviewTimeExpose,
  ScrubberPreviewTimeSlotState,
  ScrubberPreviewTrackExpose,
} from "./scrubber-preview-types.ts";
