export { createLiveRegion, useLiveRegion } from "./live-region-runtime.ts";

/** Accessible live region for status and error announcements. */
export { default as LiveRegion } from "./live-region.vue";

export type {
  LiveRegionController,
  LiveRegionOptions,
  LiveRegionPoliteness,
} from "./live-region-types.ts";
