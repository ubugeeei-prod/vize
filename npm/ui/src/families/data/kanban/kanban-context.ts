import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { DragAndDropController } from "../../interaction/drag-and-drop/drag-and-drop.ts";
import type { DropEdge } from "../../interaction/drag-and-drop/drag-and-drop-types.ts";
import type { KanbanDragData } from "./kanban-types.ts";

/** Value-erased board operations shared with columns and cards. */
export interface KanbanContextValue {
  readonly id: ComputedRef<string>;
  readonly dnd: DragAndDropController<KanbanDragData>;
  /** Card key that owns the roving tab stop. */
  readonly tabStop: ComputedRef<string | null>;
  readonly activeCard: ShallowRef<string | null>;
  readonly instructionsId: ComputedRef<string>;
  /** Whether `cardKey` may be dropped into `columnId`. */
  readonly accepts: (cardKey: string, columnId: string) => boolean;
  /** Commit a drop relative to a card target or a column target. */
  readonly drop: (cardKey: string, target: KanbanDropTarget) => void;
  /** Roving keyboard navigation from a focused card. */
  readonly navigate: (cardKey: string, event: KeyboardEvent) => void;
}

/** Destination resolved from a drop target. */
export type KanbanDropTarget =
  | { readonly kind: "card"; readonly cardKey: string; readonly edge: DropEdge | null }
  | { readonly kind: "column"; readonly columnId: string };

export const kanbanContext = createContext<KanbanContextValue>("Kanban");
