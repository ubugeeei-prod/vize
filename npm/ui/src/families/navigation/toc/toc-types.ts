/** Smooth or instant scrolling for script-driven link navigation. */
export type TocScrollBehavior = "auto" | "instant" | "smooth";

/** State exposed by the TocRoot data contract. */
export type TocState = "active" | "idle";

/** One heading collected by {@link collectTocEntries}. */
export interface TocEntry {
  /** Heading id used as the link fragment. */
  readonly id: string;

  /** Normalized heading text. */
  readonly text: string;

  /** Heading level, 1 through 6. */
  readonly level: number;
}

/** Options for {@link collectTocEntries}. */
export interface TocCollectOptions {
  /**
   * Heading selector.
   *
   * @default "h2, h3"
   */
  readonly selector?: string;

  /**
   * Skip headings marked with this attribute.
   *
   * @default "data-toc-ignore"
   */
  readonly ignoreAttribute?: string;
}

/** State exposed to the TocRoot default slot. */
export interface TocSlotState {
  /** Id of the section currently in view, or `null`. */
  readonly activeId: string | null;

  /** Link target ids in document order. */
  readonly ids: readonly string[];

  /** Stable state token. */
  readonly state: TocState;
}

/** State exposed to TocLink and TocItem slots. */
export interface TocLinkSlotState {
  /** Target heading id. */
  readonly targetId: string;

  /** Whether the target section is in view. */
  readonly active: boolean;
}

/** Public instance exposed by TocRoot. */
export interface TocRootExpose {
  /** Rendered navigation landmark. */
  readonly element: HTMLElement | null;

  /** Id of the section currently in view. */
  readonly activeId: string | null;

  /** Scroll a target into view and make it active. */
  readonly scrollTo: (id: string) => boolean;

  /** Re-measure targets immediately. */
  readonly refresh: () => void;
}
