import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { DataGridSelectIntent } from "./data-grid-model.ts";
import type {
  DataGridCellPosition,
  DataGridEditState,
  DataGridSelectionMode,
} from "./data-grid-types.ts";

/**
 * Value-erased grid operations shared with the row, cell, and header parts.
 * The generic root owns every `Row`/`Column` typed surface; parts only need ids.
 */
export interface DataGridContextValue {
  readonly id: ComputedRef<string>;
  readonly selectionMode: ComputedRef<DataGridSelectionMode>;
  readonly treeGrid: ComputedRef<boolean>;
  readonly activeCell: Readonly<ShallowRef<DataGridCellPosition | null>>;
  readonly editing: Readonly<ShallowRef<DataGridEditState | null>>;
  /** Move the roving focus to a cell and focus it. */
  readonly focusCell: (position: DataGridCellPosition) => void;
  /** Pointer selection for a clicked row (Shift ranges, Ctrl/Meta toggles). */
  readonly rowClick: (event: MouseEvent, rowId: string) => void;
  readonly select: (rowId: string, intent: DataGridSelectIntent) => boolean;
  readonly toggleExpanded: (rowId: string) => boolean;
  readonly toggleSort: (columnId: string, multi: boolean) => boolean;
  readonly resizeColumn: (columnId: string, width: number) => boolean;
  readonly startEdit: (position: DataGridCellPosition) => boolean;
  readonly setDraft: (draft: string) => void;
  readonly commitEdit: () => boolean;
  readonly cancelEdit: () => boolean;
}

export const dataGridContext = createContext<DataGridContextValue>("DataGrid");
