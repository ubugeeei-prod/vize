export { createPresence, usePresence } from "./presence-runtime.ts";

/** Accessible, unstyled presence host with enter and exit phases. */
export { default as Presence } from "./presence.vue";

export type {
  PresenceController,
  PresenceOptions,
  PresenceProps,
  PresenceStatus,
} from "./presence-types.ts";
