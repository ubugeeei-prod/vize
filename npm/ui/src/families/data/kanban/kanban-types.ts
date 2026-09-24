import type { DragAnnouncements } from "../../interaction/drag-and-drop/drag-and-drop-types.ts";

/** One board column. */
export interface KanbanColumn<ColumnId extends string> {
  /** Stable column id; also the key into the board record. */
  readonly id: ColumnId;

  /** Visible title, used as the column's accessible name. */
  readonly title: string;

  /** Refuse drops into this column. @default false */
  readonly disabled?: boolean;

  /** Maximum number of cards (WIP limit); full columns refuse cards from other columns. */
  readonly limit?: number;
}

/** Cards per column. */
export type KanbanBoard<Card, ColumnId extends string> = {
  readonly [Id in ColumnId]: readonly Card[];
};

/** A card position on the board. */
export interface KanbanLocation<ColumnId extends string> {
  /** Column id. */
  readonly columnId: ColumnId;

  /** Zero-based index within the column. */
  readonly index: number;
}

/** Emitted after a card move commits. */
export interface KanbanMoveEvent<Card, ColumnId extends string> {
  /** Moved card. */
  readonly card: Card;

  /** Moved card key. */
  readonly cardKey: string;

  /** Origin position. */
  readonly from: KanbanLocation<ColumnId>;

  /** Destination position in the resulting board. */
  readonly to: KanbanLocation<ColumnId>;

  /** Board after the move. */
  readonly board: KanbanBoard<Card, ColumnId>;
}

/** Slot props for a column header. */
export interface KanbanColumnSlotProps<Card, ColumnId extends string> {
  /** Column definition. */
  readonly column: KanbanColumn<ColumnId>;

  /** Cards in the column. */
  readonly cards: readonly Card[];

  /** Whether the column reached its limit. */
  readonly full: boolean;
}

/** Slot props for a card. */
export interface KanbanCardSlotProps<Card, ColumnId extends string> {
  /** Consumer card. */
  readonly card: Card;

  /** Card key. */
  readonly cardKey: string;

  /** Column the card is in. */
  readonly column: KanbanColumn<ColumnId>;

  /** Zero-based index in the column. */
  readonly index: number;

  /** Whether this card is being dragged. */
  readonly dragging: boolean;
}

/** Card payload data carried by the drag-and-drop session. */
export type KanbanDragData = string;

/** Announcement overrides for grab, move, drop, and cancel. */
export type KanbanAnnouncements = DragAnnouncements<KanbanDragData>;

/** Imperative surface exposed by Kanban. */
export interface KanbanExpose {
  /** Rendered board element. */
  readonly element: HTMLDivElement | null;

  /** Focus a card by key (or the current tab stop). */
  readonly focusCard: (cardKey?: string) => void;

  /** Cancel an active drag. */
  readonly cancelDrag: () => boolean;
}
