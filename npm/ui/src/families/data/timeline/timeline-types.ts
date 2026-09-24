/** Layout axis of a timeline. */
export type TimelineOrientation = "horizontal" | "vertical";

/** Progress status of one timeline item. */
export type TimelineItemStatus = "complete" | "current" | "upcoming";

/** State exposed to the TimelineRoot default slot. */
export interface TimelineSlotState {
  /** Value of the current item, or `null` when progress is not tracked. */
  readonly value: string | null;

  /** Layout axis. */
  readonly orientation: TimelineOrientation;

  /** Whether items are listed newest first. */
  readonly reversed: boolean;

  /** Number of registered items. */
  readonly count: number;
}

/** State exposed to TimelineItem and its parts. */
export interface TimelineItemSlotState {
  /** Item value, or `null` for untracked items. */
  readonly value: string | null;

  /** Zero-based position in document order. */
  readonly index: number;

  /** Progress status: explicit, derived from the root value, or `null`. */
  readonly status: TimelineItemStatus | null;

  /** Whether this is the last item, which usually omits its connector. */
  readonly last: boolean;
}

/** Public instance exposed by TimelineRoot. */
export interface TimelineRootExpose {
  /** Rendered ordered list. */
  readonly element: HTMLOListElement | null;

  /** Current item value. */
  readonly value: string | null;

  /** Registered item values in document order. */
  readonly values: readonly string[];
}

/** Public instance exposed by TimelineItem. */
export interface TimelineItemExpose extends TimelineItemSlotState {
  /** Rendered list item. */
  readonly element: HTMLLIElement | null;
}
