import type { DragSourceProps } from "../../interaction/drag-and-drop/drag-and-drop-controller-types.ts";

/** Row selection policy. */
export type GridListSelectionMode = "multiple" | "none" | "single";

/** Spatial layout driving arrow-key geometry. */
export type GridListLayout = "grid" | "list";

/** Emitted after a drag or keyboard reorder commits. */
export interface GridListReorderEvent<Item> {
  /** Moved item. */
  readonly item: Item;

  /** Moved item key. */
  readonly key: string;

  /** Zero-based origin index. */
  readonly fromIndex: number;

  /** Zero-based destination index. */
  readonly toIndex: number;

  /** Items in their new order. */
  readonly items: readonly Item[];
}

/** Props for the `item` slot. */
export interface GridListItemSlotProps<Item> {
  /** Consumer item. */
  readonly item: Item;

  /** Stable item key. */
  readonly key: string;

  /** Zero-based index. */
  readonly index: number;

  /** Whether the item is selected. */
  readonly selected: boolean;

  /** Whether the item owns the roving focus. */
  readonly active: boolean;

  /** Whether the item is disabled. */
  readonly disabled: boolean;

  /** Whether the item is being dragged. */
  readonly dragging: boolean;

  /**
   * Handlers to spread on a drag handle (`v-bind="dragHandleProps"`), or `null`
   * when reordering is off. Enter or Space on the handle starts a keyboard move.
   */
  readonly dragHandleProps: Readonly<DragSourceProps> | null;
}

/** Imperative surface exposed by GridList. */
export interface GridListExpose {
  /** Rendered `role="grid"` element. */
  readonly element: HTMLDivElement | null;

  /** Focus an item by key (or the active/first item). */
  readonly focus: (key?: string) => void;

  /** Select every enabled item (multiple selection only). */
  readonly selectAll: () => boolean;

  /** Clear the selection. */
  readonly clearSelection: () => boolean;
}
