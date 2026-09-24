import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Why the active target changed. */
export type ScrollSpyChangeReason = "navigation" | "scroll";

/** Options shared by {@link createScrollSpy} and {@link useScrollSpy}. */
export interface ScrollSpyOptions {
  /** Ids of observed target elements, in document order. */
  readonly ids: MaybeRefOrGetter<readonly string[]>;

  /**
   * Scroll container whose top edge anchors the activation line. `null` and
   * `undefined` select the document viewport.
   *
   * @default undefined
   */
  readonly root?: MaybeRefOrGetter<Element | null | undefined>;

  /**
   * Distance in CSS pixels below the container top where a target becomes
   * active, for example the height of a sticky header.
   *
   * @default 0
   */
  readonly offset?: MaybeRefOrGetter<number | undefined>;

  /**
   * Active id assumed before the first measurement, which is what server
   * rendering and hydration use.
   *
   * @default null
   */
  readonly initialActiveId?: string | null;

  /**
   * Stop tracking while true. The last active id is kept.
   *
   * @default false
   */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;

  /** Called after a distinct active-id change. */
  readonly onActiveChange?: (
    id: string | null,
    previous: string | null,
    reason: ScrollSpyChangeReason,
  ) => void;
}

/** Scroll position tracker for a set of in-page targets. */
export interface ScrollSpyController {
  /** Id of the last target whose top crossed the activation line, or `null`. */
  readonly activeId: Readonly<ShallowRef<string | null>>;

  /** Ids of targets currently intersecting the container, in document order. */
  readonly visibleIds: Readonly<ShallowRef<readonly string[]>>;

  /** Measure targets now instead of on the next animation frame. */
  readonly refresh: () => void;

  /** Scroll a target into view and make it active immediately. */
  readonly scrollTo: (id: string, options?: ScrollIntoViewOptions) => boolean;

  /** Remove listeners and stop tracking. Safe to repeat. */
  readonly dispose: () => void;
}
