import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** ARIA live politeness used by the announcer. */
export type LiveRegionPoliteness = "assertive" | "polite";

/** Options shared by {@link createLiveRegion} and {@link useLiveRegion}. */
export interface LiveRegionOptions {
  /**
   * Default announcement urgency.
   *
   * @default "polite"
   */
  readonly politeness?: MaybeRefOrGetter<LiveRegionPoliteness | undefined>;
}

/** Stateful live-region announcer with an explicit queue. */
export interface LiveRegionController {
  /** Text currently exposed to assistive technology. */
  readonly message: Readonly<ShallowRef<string>>;

  /** Active politeness for the rendered live region. */
  readonly politeness: Readonly<ShallowRef<LiveRegionPoliteness>>;

  /** Queue an announcement. Identical text is re-announced by clearing first. */
  readonly announce: (text: string, politeness?: LiveRegionPoliteness) => void;

  /** Clear the current announcement. */
  readonly clear: () => void;

  /** Release pending announcements. */
  readonly dispose: () => void;
}
