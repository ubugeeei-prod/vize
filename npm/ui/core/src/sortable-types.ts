import type { MaybeRefOrGetter, ShallowRef } from "vue";

import type { DragSourceProps } from "./drag-and-drop-controller-types.ts";
import type { DragPointerType, DropTargetRect } from "./drag-and-drop-types.ts";

/** Spatial arrangement of one sortable container. */
export type SortableOrientation = "grid" | "horizontal" | "vertical";

/** Resolved writing direction used by horizontal and grid arrow keys. */
export type SortableDirection = "ltr" | "rtl";

/** Drop position relative to the item under the drag. */
export type SortablePosition = "after" | "before" | "inside";

/** Lifecycle phases emitted by a sortable controller. */
export type SortableEventType = "sortcancel" | "sortcommit" | "sortpreview" | "sortstart";

/** Immutable snapshot shared by every sortable lifecycle phase. */
export interface SortableEvent<Type extends SortableEventType = SortableEventType> {
  /** Lifecycle phase represented by this immutable snapshot. */
  readonly type: Type;

  /** Input family that owns this sort. */
  readonly pointerType: DragPointerType;

  /** Key of the item being sorted. */
  readonly key: string;

  /**
   * Zero-based index the move starts from. `sortstart`, `sortpreview`, and
   * `sortcancel` report the item's current index; `sortcommit` reports the
   * index captured when the item was picked up.
   */
  readonly fromIndex: number;

  /**
   * Zero-based destination index. `sortcancel` reports the index the item must
   * return to; an `"inside"` move reports the receiving item's index.
   */
  readonly toIndex: number;

  /** Key of the item the move is relative to, or `null` for index-only moves. */
  readonly overKey: string | null;

  /** Position relative to `overKey`, or `null` for index-only moves. */
  readonly position: SortablePosition | null;

  /** Native event responsible for this phase, or `null` for manual settlement. */
  readonly originalEvent: Event | null;
}

/** Snapshot emitted once when an item is picked up. */
export type SortStartEvent = SortableEvent<"sortstart">;

/** Snapshot emitted when the projected destination changes before a drop. */
export type SortPreviewEvent = SortableEvent<"sortpreview">;

/** Snapshot emitted when a completed drop commits the move. */
export type SortCommitEvent = SortableEvent<"sortcommit">;

/** Snapshot emitted when a sort is canceled and the item must return. */
export type SortCancelEvent = SortableEvent<"sortcancel">;

/** Reactive indicator state for the projected destination. */
export interface SortableIndicatorState {
  /** Key of the item the indicator is relative to. */
  readonly key: string;

  /** Position relative to `key`. */
  readonly position: SortablePosition;

  /** Zero-based projected destination index. */
  readonly toIndex: number;

  /** Rectangle of the reference item, or `null` when unmeasurable. */
  readonly rect: DropTargetRect | null;

  /** Zero-thickness line along the drop edge; `null` for `"inside"` drops. */
  readonly line: DropTargetRect | null;
}

/** Context handed to announcement builders before speaking to assistive tech. */
export interface SortableAnnouncementContext {
  readonly pointerType: DragPointerType;
  readonly key: string;
  readonly label: string;

  /** Zero-based indexes; built-in messages render them one-based. */
  readonly fromIndex: number;
  readonly toIndex: number;
  readonly count: number;
  readonly overKey: string | null;
  readonly overLabel: string | null;
  readonly position: SortablePosition | null;
}

/** Localizable builders for grab, move, drop, and cancel announcements. */
export interface SortableAnnouncements {
  /** Spoken when an item is picked up. Return `null` to stay silent. */
  readonly grab?: (context: SortableAnnouncementContext) => string | null;

  /** Spoken when the projected destination changes. */
  readonly move?: (context: SortableAnnouncementContext) => string | null;

  /** Spoken after a committed drop. */
  readonly drop?: (context: SortableAnnouncementContext) => string | null;

  /** Spoken after a canceled sort. */
  readonly cancel?: (context: SortableAnnouncementContext) => string | null;
}

/** Options shared by `createSortable` and `useSortable`. */
export interface SortableOptions {
  /**
   * Spatial arrangement deciding drop edges and arrow-key geometry.
   *
   * @default "vertical"
   */
  readonly orientation?: MaybeRefOrGetter<SortableOrientation | undefined>;

  /**
   * Column count used by grid arrow-key geometry.
   * The resolved value must be a finite integer greater than zero.
   *
   * @default 1
   */
  readonly columns?: MaybeRefOrGetter<number | undefined>;

  /**
   * Writing direction that flips horizontal and grid arrow keys.
   *
   * @default "ltr"
   */
  readonly direction?: MaybeRefOrGetter<SortableDirection | undefined>;

  /**
   * Allow `"inside"` drops so nested trees can re-parent items.
   *
   * @default false
   */
  readonly nesting?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Suppress new sorts and cancel the active sort on its next event.
   *
   * @default false
   */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Chebyshev distance in CSS pixels a pointer must travel before a sort
   * starts. The resolved value must be finite and greater than or equal to zero.
   *
   * @default 4
   */
  readonly startDistance?: MaybeRefOrGetter<number | undefined>;

  /**
   * Overrides merged over the built-in English announcement builders.
   *
   * @default built-in English announcements
   */
  readonly announcements?: SortableAnnouncements;

  /** Called exactly once when an item is picked up. */
  readonly onSortStart?: (event: SortStartEvent) => void;

  /** Called when the projected destination changes before a drop. */
  readonly onSortPreview?: (event: SortPreviewEvent) => void;

  /** Called exactly once when a completed drop commits the move. */
  readonly onSortCommit?: (event: SortCommitEvent) => void;

  /** Called exactly once when a sort is canceled; the item must return. */
  readonly onSortCancel?: (event: SortCancelEvent) => void;
}

/** Declaration of one sortable item owned by a controller. */
export interface SortableItemOptions {
  /** Stable key unique among this controller's items. */
  readonly key: string;

  /** Element that owns this item's geometry and document order. */
  readonly element: MaybeRefOrGetter<Element | null | undefined>;

  /**
   * Viewport rectangle used for hit testing and indicator geometry.
   *
   * @default the element's `getBoundingClientRect()`
   */
  readonly getRect?: () => DropTargetRect | DOMRectReadOnly | null | undefined;

  /**
   * Human-readable name used by announcements.
   *
   * @default the item key
   */
  readonly label?: MaybeRefOrGetter<string | undefined>;

  /**
   * Suppress sorting this item; it remains a valid drop reference.
   *
   * @default false
   */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;
}

/** Live registration owned by one sortable item. */
export interface SortableItemRegistration {
  /** Key this registration was created with. */
  readonly key: string;

  /** Whether this item owns the active sort. */
  readonly isDragging: Readonly<ShallowRef<boolean>>;

  /** Stable native handlers to spread or merge onto one item or its handle. */
  readonly itemProps: Readonly<DragSourceProps>;

  /** Remove the item; its active sort is canceled first. */
  readonly dispose: () => void;
}

/** Stateful sortable coordinator with explicit ownership and disposal. */
export interface SortableController {
  /** Whether a pointer or keyboard sort is active. */
  readonly isSorting: Readonly<ShallowRef<boolean>>;

  /** Key of the item that owns the active sort, or `null`. */
  readonly activeKey: Readonly<ShallowRef<string | null>>;

  /** Indicator state for the projected destination, or `null`. */
  readonly indicator: Readonly<ShallowRef<SortableIndicatorState | null>>;

  /** Register one sortable item. Duplicate keys are rejected. */
  readonly registerItem: (options: SortableItemOptions) => SortableItemRegistration;

  /**
   * Cancel the active sort, including one that has not started yet.
   *
   * @returns `true` when an owned sort was settled.
   * @throws An error carrying `VIZE_UI_SORTABLE_DISPOSED` after disposal.
   */
  readonly cancel: () => boolean;

  /** Release listeners, the live region, and reactive state without callbacks. */
  readonly dispose: () => void;
}
