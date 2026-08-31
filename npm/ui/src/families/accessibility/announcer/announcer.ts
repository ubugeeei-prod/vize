export {
  announcerContext,
  createAnnouncer,
  useAnnouncer,
  useAnnouncerOwnership,
} from "./announcer-runtime.ts";

export { createBusyAnnouncement, useBusyAnnouncement } from "./announcer-busy.ts";

/** Document announcer that owns the deduplicated polite and assertive live regions. */
export { default as AnnouncerProvider } from "./announcer-provider.vue";

export type {
  AnnouncerController,
  AnnouncerMessageOptions,
  AnnouncerOptions,
  AnnouncerOwnership,
  AnnouncerPoliteness,
  BusyAnnouncement,
  BusyAnnouncementOptions,
} from "./announcer-types.ts";
