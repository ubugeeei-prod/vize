import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { CollectionRegistry } from "../../foundations/collection/collection.ts";
import type { DragSourceProps } from "../../interaction/drag-and-drop/drag-and-drop-controller-types.ts";
import type { SortableController } from "../../interaction/sortable/sortable.ts";
import type { GridListSelectionMode } from "./grid-list-types.ts";

/** Value-erased list state shared with GridListItem. */
export interface GridListContextValue {
  readonly id: ComputedRef<string>;
  readonly registry: CollectionRegistry<string, number>;
  readonly selectionMode: ComputedRef<GridListSelectionMode>;
  readonly sortable: SortableController | null;
  /** Drag handle props published by each item for the typed slot in the root. */
  readonly handles: ReadonlyMap<string, Readonly<DragSourceProps>>;
  readonly publishHandle: (key: string, props: Readonly<DragSourceProps> | null) => void;
  readonly onItemKeydown: (key: string, event: KeyboardEvent) => void;
  readonly onItemClick: (key: string, event: MouseEvent) => void;
}

export const gridListContext = createContext<GridListContextValue>("GridList");
