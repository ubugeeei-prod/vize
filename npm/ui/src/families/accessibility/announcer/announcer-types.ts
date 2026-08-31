import type { MaybeRefOrGetter, ShallowRef } from "vue";

import type { LiveRegionPoliteness } from "../live-region/live-region-types.ts";

/** Announcement urgency accepted by the announcer queue. */
export type AnnouncerPoliteness = LiveRegionPoliteness;

/** Options shared by {@link createAnnouncer} and {@link useAnnouncer}. */
export interface AnnouncerOptions {
  /**
   * Default urgency for announcements that do not name one.
   *
   * @default "polite"
   */
  readonly politeness?: MaybeRefOrGetter<AnnouncerPoliteness | undefined>;
}

/** Per-announcement options accepted by {@link AnnouncerController.announce}. */
export interface AnnouncerMessageOptions {
  /**
   * Urgency channel for this announcement.
   *
   * @default The announcer's default politeness
   */
  readonly politeness?: AnnouncerPoliteness;

  /**
   * Coalescing key. A pending announcement with the same key is replaced by
   * this one instead of speaking both.
   *
   * @default undefined
   */
  readonly key?: string;
}

/** Document-wide announcement queue with coalescing and deduplication. */
export interface AnnouncerController {
  /** Text currently exposed on the polite channel. */
  readonly politeMessage: Readonly<ShallowRef<string>>;

  /** Text currently exposed on the assertive channel. */
  readonly assertiveMessage: Readonly<ShallowRef<string>>;

  /** Announcements queued but not yet exposed to assistive technology. */
  readonly pendingCount: Readonly<ShallowRef<number>>;

  /**
   * Queue one announcement.
   *
   * Text identical to a pending or in-flight announcement on the same channel
   * is dropped, a pending announcement with the same key is replaced, and
   * assertive announcements precede queued polite announcements.
   *
   * @returns `false` when the announcement was deduplicated.
   */
  readonly announce: (text: string, options?: AnnouncerMessageOptions) => boolean;

  /** Remove one pending keyed announcement before it is spoken. */
  readonly cancel: (key: string) => boolean;

  /** Drop pending announcements and clear both channels. */
  readonly clear: () => void;

  /** Release the queue and both live-region channels. */
  readonly dispose: () => void;
}

/** Ownership resolution for one AnnouncerProvider instance. */
export interface AnnouncerOwnership {
  /** The document announcer this subtree should use. */
  readonly announcer: AnnouncerController;

  /** Whether this provider owns the rendered live regions. */
  readonly isOwner: boolean;
}

/** Options accepted by {@link createBusyAnnouncement} and {@link useBusyAnnouncement}. */
export interface BusyAnnouncementOptions {
  /** Announced when the tracked work begins. */
  readonly label: string;

  /**
   * Urgency for the begin and progress announcements.
   *
   * @default "polite"
   */
  readonly politeness?: AnnouncerPoliteness;
}

/** Announcement policy for one loading, streaming, or background task. */
export interface BusyAnnouncement {
  /** Whether the tracked work is still running. Bind `aria-busy` to this. */
  readonly isBusy: Readonly<ShallowRef<boolean>>;

  /** Report progress. Progress updates coalesce so only the latest is spoken. */
  readonly update: (text: string) => void;

  /** Finish: cancel unspoken progress, then optionally announce an outcome. */
  readonly end: (text?: string) => void;
}
