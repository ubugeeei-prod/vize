import { computed, shallowRef, toValue } from "vue";
import type { ComputedRef, MaybeRefOrGetter, ShallowRef } from "vue";

import { useControllableState } from "../../foundations/controllable-state/controllable-state.ts";
import {
  cellValue,
  columnCallbacks,
  formatCell,
  isEditable,
  isSortable,
} from "./data-grid-columns.ts";
import { buildRowModels, toggleSorting } from "./data-grid-rows.ts";
import type {
  DataGridCellEditEvent,
  DataGridCellPosition,
  DataGridColumn,
  DataGridColumnModel,
  DataGridEditState,
  DataGridPinning,
  DataGridPinSide,
  DataGridRowModel,
  DataGridSelectionMode,
  DataGridSort,
} from "./data-grid-types.ts";

/** Every piece of user-changeable grid state; each slice can be controlled. */
export interface DataGridState {
  /** Sort keys, highest priority first. */
  readonly sorting: readonly DataGridSort[];
  /** Text matched against every value column. */
  readonly globalFilter: string;
  /** Per-column filter values handed to each column's `filter`. */
  readonly columnFilters: Readonly<Record<string, unknown>>;
  /** Column visibility overrides by id. */
  readonly columnVisibility: Readonly<Record<string, boolean>>;
  /** Column ids in display order (pinning groups are applied on top). */
  readonly columnOrder: readonly string[];
  /** Pinned column ids per side. */
  readonly columnPinning: DataGridPinning;
  /** Column widths by id in CSS pixels. */
  readonly columnSizing: Readonly<Record<string, number>>;
  /** Selected row ids. */
  readonly selection: readonly string[];
  /** Expanded row ids. */
  readonly expanded: readonly string[];
}

/** Name of one state slice. */
export type DataGridStateKey = keyof DataGridState;

/** Reactive inputs for {@link useDataGrid}. */
export interface DataGridOptions<Row, Column extends DataGridColumn<Row>> {
  /** Source rows. */
  readonly rows: MaybeRefOrGetter<readonly Row[]>;
  /** Column definitions. */
  readonly columns: MaybeRefOrGetter<readonly Column[]>;
  /** Stable row id. @default index path such as `"3"` or `"3.1"` */
  readonly getRowId?: (row: Row, index: number, parentId: string | null) => string;
  /** Sub rows for tree grids. @default undefined */
  readonly getSubRows?: (row: Row) => readonly Row[] | null | undefined;
  /** Render tree-grid semantics. @default `getSubRows !== undefined` */
  readonly treeGrid?: MaybeRefOrGetter<boolean | undefined>;
  /** Extra row predicate applied with the column and global filters. @default undefined */
  readonly filterRow?: MaybeRefOrGetter<((row: Row) => boolean) | undefined>;
  /** Row selection policy. @default "none" */
  readonly selectionMode?: MaybeRefOrGetter<DataGridSelectionMode | undefined>;
  /** Rows moved by PageUp/PageDown. @default 10 */
  readonly pageSize?: MaybeRefOrGetter<number | undefined>;
  /** Reading direction for horizontal arrow keys. @default "ltr" */
  readonly dir?: MaybeRefOrGetter<"ltr" | "rtl" | undefined>;
  /** Controlled state slices; `undefined` leaves a slice uncontrolled. */
  readonly state?: {
    readonly [Key in DataGridStateKey]?: MaybeRefOrGetter<DataGridState[Key] | undefined>;
  };
  /** Initial uncontrolled state slices. */
  readonly defaultState?: MaybeRefOrGetter<Partial<DataGridState> | undefined>;
  /** Called after any slice changes (controlled or not). */
  readonly onStateChange?: <Key extends DataGridStateKey>(
    key: Key,
    value: DataGridState[Key],
  ) => void;
  /** Called when an edit commits. The grid never mutates rows. */
  readonly onCellEdit?: (event: DataGridCellEditEvent<Row>) => void;
}

/** How a row selection request combines with the current selection. */
export type DataGridSelectIntent = "range" | "replace" | "toggle";

/** Headless data-grid model returned by {@link useDataGrid}. */
export interface DataGridController<Row, Column extends DataGridColumn<Row>> {
  readonly state: ComputedRef<DataGridState>;
  readonly rowModels: ComputedRef<readonly DataGridRowModel<Row>[]>;
  readonly columnModels: ComputedRef<readonly DataGridColumnModel<Row, Column>[]>;
  readonly totalWidth: ComputedRef<number>;
  readonly allSelected: ComputedRef<boolean>;
  readonly someSelected: ComputedRef<boolean>;
  readonly treeGrid: ComputedRef<boolean>;
  readonly selectionMode: ComputedRef<DataGridSelectionMode>;
  readonly activeCell: Readonly<ShallowRef<DataGridCellPosition | null>>;
  readonly editing: Readonly<ShallowRef<DataGridEditState | null>>;
  readonly setState: <Key extends DataGridStateKey>(key: Key, value: DataGridState[Key]) => boolean;
  readonly toggleSort: (columnId: string, multi?: boolean) => boolean;
  readonly setGlobalFilter: (value: string) => boolean;
  readonly setColumnFilter: (columnId: string, value: unknown) => boolean;
  readonly setColumnVisible: (columnId: string, visible: boolean) => boolean;
  readonly moveColumn: (columnId: string, toIndex: number) => boolean;
  readonly pinColumn: (columnId: string, side: DataGridPinSide | null) => boolean;
  readonly resizeColumn: (columnId: string, width: number) => boolean;
  readonly select: (rowId: string, intent?: DataGridSelectIntent) => boolean;
  readonly toggleAllSelected: () => boolean;
  readonly toggleExpanded: (rowId: string, expanded?: boolean) => boolean;
  readonly setActiveCell: (position: DataGridCellPosition | null) => void;
  readonly startEdit: (position?: DataGridCellPosition) => boolean;
  readonly setDraft: (draft: string) => void;
  readonly commitEdit: () => boolean;
  readonly cancelEdit: () => boolean;
  readonly cellValue: (row: Row, column: Column) => unknown;
  readonly cellText: (row: Row, column: Column) => string;
  /** Apply APG grid keyboard behavior; returns whether the key was handled. */
  readonly handleKeydown: (event: KeyboardEvent) => boolean;
}

const defaultWidth = 150;
const defaultMinWidth = 40;

function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

function sameIds(left: readonly string[], right: readonly string[]): boolean {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

/**
 * Create the headless model behind `DataGrid`: row pipeline, column layout,
 * selection, expansion, editing, and WAI-ARIA APG grid keyboard behavior.
 * Every state slice is controllable, so the SFC and fully custom renderers share it.
 */
export function useDataGrid<Row, Column extends DataGridColumn<Row>>(
  options: DataGridOptions<Row, Column>,
): DataGridController<Row, Column> {
  const columns = computed(() => toValue(options.columns));
  const selectionMode = computed(() => toValue(options.selectionMode) ?? "none");
  const dir = computed(() => toValue(options.dir) ?? "ltr");
  const initialColumns = toValue(options.columns);

  function slice<Key extends DataGridStateKey>(key: Key, fallback: () => DataGridState[Key]) {
    return useControllableState<DataGridState[Key]>({
      value: () => toValue(options.state?.[key]),
      defaultValue: () => toValue(options.defaultState)?.[key] ?? fallback(),
      onChange: (value) => options.onStateChange?.(key, value),
    });
  }

  const slices = {
    sorting: slice("sorting", () => []),
    globalFilter: slice("globalFilter", () => ""),
    columnFilters: slice("columnFilters", () => ({})),
    columnVisibility: slice("columnVisibility", () =>
      Object.fromEntries(initialColumns.filter((c) => c.hidden).map((c) => [c.id, false])),
    ),
    columnOrder: slice("columnOrder", () => initialColumns.map((column) => column.id)),
    columnPinning: slice("columnPinning", () => ({
      start: initialColumns.filter((c) => c.pin === "start").map((c) => c.id),
      end: initialColumns.filter((c) => c.pin === "end").map((c) => c.id),
    })),
    columnSizing: slice("columnSizing", () => ({})),
    selection: slice("selection", () => []),
    expanded: slice("expanded", () => []),
  };

  const state = computed<DataGridState>(() => ({
    sorting: slices.sorting.value.value,
    globalFilter: slices.globalFilter.value.value,
    columnFilters: slices.columnFilters.value.value,
    columnVisibility: slices.columnVisibility.value.value,
    columnOrder: slices.columnOrder.value.value,
    columnPinning: slices.columnPinning.value.value,
    columnSizing: slices.columnSizing.value.value,
    selection: slices.selection.value.value,
    expanded: slices.expanded.value.value,
  }));

  const setters: { readonly [Key in DataGridStateKey]: (value: DataGridState[Key]) => boolean } = {
    sorting: slices.sorting.set,
    globalFilter: slices.globalFilter.set,
    columnFilters: slices.columnFilters.set,
    columnVisibility: slices.columnVisibility.set,
    columnOrder: slices.columnOrder.set,
    columnPinning: slices.columnPinning.set,
    columnSizing: slices.columnSizing.set,
    selection: slices.selection.set,
    expanded: slices.expanded.set,
  };

  function setState<Key extends DataGridStateKey>(key: Key, value: DataGridState[Key]): boolean {
    const set: (next: DataGridState[Key]) => boolean = setters[key];
    return set(value);
  }

  const getRowId =
    options.getRowId ??
    ((_row: Row, index: number, parentId: string | null) =>
      parentId === null ? String(index) : `${parentId}.${index}`);

  const rowModels = computed(() =>
    buildRowModels<Row>({
      rows: toValue(options.rows),
      columns: columns.value,
      getRowId,
      getSubRows: options.getSubRows,
      sorting: state.value.sorting,
      globalFilter: state.value.globalFilter,
      columnFilters: state.value.columnFilters,
      filterRow: toValue(options.filterRow),
      expanded: new Set(state.value.expanded),
      selected: new Set(state.value.selection),
    }),
  );
  const treeGrid = computed(() => toValue(options.treeGrid) ?? options.getSubRows !== undefined);

  function widthOf(column: Column): number {
    const sized = state.value.columnSizing[column.id];
    return clamp(
      sized ?? column.width ?? defaultWidth,
      column.minWidth ?? defaultMinWidth,
      column.maxWidth ?? Number.POSITIVE_INFINITY,
    );
  }

  const columnModels = computed<readonly DataGridColumnModel<Row, Column>[]>(() => {
    const byId = new Map(columns.value.map((column) => [column.id, column]));
    const order = [
      ...state.value.columnOrder.filter((id) => byId.has(id)),
      ...columns.value
        .map((column) => column.id)
        .filter((id) => !state.value.columnOrder.includes(id)),
    ];
    const visible = order.filter((id) => state.value.columnVisibility[id] !== false);
    const { start, end } = state.value.columnPinning;
    const startIds = start.filter((id) => visible.includes(id));
    const endIds = end.filter((id) => visible.includes(id) && !startIds.includes(id));
    const center = visible.filter((id) => !startIds.includes(id) && !endIds.includes(id));
    const ordered = [...startIds, ...center, ...endIds];
    const sorting = state.value.sorting;
    let startOffset = 0;
    const endOffsets = new Map<string, number>();
    let endOffset = 0;
    for (const id of [...endIds].reverse()) {
      endOffsets.set(id, endOffset);
      const column = byId.get(id);
      if (column) endOffset += widthOf(column);
    }
    return ordered.flatMap((id, index) => {
      const column = byId.get(id);
      if (!column) return [];
      const pin: DataGridPinSide | null = startIds.includes(id)
        ? "start"
        : endIds.includes(id)
          ? "end"
          : null;
      const width = widthOf(column);
      const pinOffset =
        pin === "start" ? startOffset : pin === "end" ? (endOffsets.get(id) ?? 0) : 0;
      if (pin === "start") startOffset += width;
      const sortIndex = sorting.findIndex((sort) => sort.columnId === id);
      return [
        Object.freeze({
          column,
          id,
          index,
          width,
          pin,
          pinOffset,
          sortDirection: sortIndex === -1 ? null : (sorting[sortIndex]?.direction ?? null),
          sortIndex: sortIndex === -1 ? null : sortIndex + 1,
          sortable: isSortable(column),
        }),
      ];
    });
  });
  const totalWidth = computed(() =>
    columnModels.value.reduce((total, column) => total + column.width, 0),
  );
  const allSelected = computed(
    () => rowModels.value.length > 0 && rowModels.value.every((row) => row.selected),
  );
  const someSelected = computed(
    () => !allSelected.value && rowModels.value.some((row) => row.selected),
  );

  const activeCell = shallowRef<DataGridCellPosition | null>(null);
  const editing = shallowRef<DataGridEditState | null>(null);
  let anchorRowId: string | null = null;

  function findColumn(id: string): Column | undefined {
    return columns.value.find((column) => column.id === id);
  }

  function findRow(id: string): DataGridRowModel<Row> | undefined {
    return rowModels.value.find((row) => row.id === id);
  }

  function toggleSort(columnId: string, multi = false): boolean {
    const column = findColumn(columnId);
    if (!column || !isSortable(column)) return false;
    return setState("sorting", toggleSorting(state.value.sorting, columnId, multi));
  }

  function setColumnFilter(columnId: string, value: unknown): boolean {
    const next = { ...state.value.columnFilters };
    if (value === undefined || value === null || value === "") delete next[columnId];
    else next[columnId] = value;
    return setState("columnFilters", next);
  }

  function setColumnVisible(columnId: string, visible: boolean): boolean {
    const column = findColumn(columnId);
    if (!column || (!visible && column.hideable === false)) return false;
    return setState("columnVisibility", { ...state.value.columnVisibility, [columnId]: visible });
  }

  function moveColumn(columnId: string, toIndex: number): boolean {
    const column = findColumn(columnId);
    if (!column || column.reorderable === false) return false;
    const order = columnModels.value.map((model) => model.id);
    const hidden = state.value.columnOrder.filter((id) => !order.includes(id));
    const from = order.indexOf(columnId);
    if (from === -1) return false;
    const target = clamp(Math.trunc(toIndex), 0, order.length - 1);
    if (target === from) return false;
    order.splice(from, 1);
    order.splice(target, 0, columnId);
    const next = [...order, ...hidden];
    return sameIds(next, state.value.columnOrder) ? false : setState("columnOrder", next);
  }

  function pinColumn(columnId: string, side: DataGridPinSide | null): boolean {
    if (!findColumn(columnId)) return false;
    const start = state.value.columnPinning.start.filter((id) => id !== columnId);
    const end = state.value.columnPinning.end.filter((id) => id !== columnId);
    if (side === "start") start.push(columnId);
    if (side === "end") end.unshift(columnId);
    return setState("columnPinning", { start, end });
  }

  function resizeColumn(columnId: string, width: number): boolean {
    const column = findColumn(columnId);
    if (!column || column.resizable === false || !Number.isFinite(width)) return false;
    const next = clamp(
      Math.round(width),
      column.minWidth ?? defaultMinWidth,
      column.maxWidth ?? Number.POSITIVE_INFINITY,
    );
    if (state.value.columnSizing[columnId] === next) return false;
    return setState("columnSizing", { ...state.value.columnSizing, [columnId]: next });
  }

  function select(rowId: string, intent: DataGridSelectIntent = "toggle"): boolean {
    const mode = selectionMode.value;
    if (mode === "none" || !findRow(rowId)) return false;
    const current = state.value.selection;
    let next: string[];
    if (mode === "single") {
      next = current.includes(rowId) && intent === "toggle" ? [] : [rowId];
    } else if (intent === "range" && anchorRowId !== null) {
      const ids = rowModels.value.map((row) => row.id);
      const from = ids.indexOf(anchorRowId);
      const to = ids.indexOf(rowId);
      const range = from === -1 ? [rowId] : ids.slice(Math.min(from, to), Math.max(from, to) + 1);
      next = [...new Set([...current, ...range])];
    } else if (intent === "replace") {
      next = [rowId];
    } else {
      next = current.includes(rowId) ? current.filter((id) => id !== rowId) : [...current, rowId];
    }
    if (intent !== "range") anchorRowId = rowId;
    return sameIds(next, current) ? false : setState("selection", next);
  }

  function toggleAllSelected(): boolean {
    if (selectionMode.value !== "multiple") return false;
    return setState("selection", allSelected.value ? [] : rowModels.value.map((row) => row.id));
  }

  function toggleExpanded(rowId: string, expanded?: boolean): boolean {
    const row = findRow(rowId);
    if (!row?.expandable) return false;
    const next = expanded ?? !row.expanded;
    if (next === row.expanded) return false;
    const current = state.value.expanded;
    return setState("expanded", next ? [...current, rowId] : current.filter((id) => id !== rowId));
  }

  function setActiveCell(position: DataGridCellPosition | null): void {
    activeCell.value = position;
  }

  function startEdit(position = activeCell.value ?? undefined): boolean {
    if (!position || position.rowId === null) return false;
    const row = findRow(position.rowId);
    const column = findColumn(position.columnId);
    if (!row || !column || !isEditable(column, row.row)) return false;
    activeCell.value = position;
    editing.value = Object.freeze({
      rowId: position.rowId,
      columnId: position.columnId,
      draft: formatCell(column, row.row),
      error: null,
    });
    return true;
  }

  function setDraft(draft: string): void {
    if (editing.value) editing.value = Object.freeze({ ...editing.value, draft, error: null });
  }

  function commitEdit(): boolean {
    const edit = editing.value;
    if (!edit) return false;
    const row = findRow(edit.rowId);
    const column = findColumn(edit.columnId);
    const callbacks = column ? columnCallbacks(column) : null;
    if (!row || !column || !callbacks) {
      editing.value = null;
      return false;
    }
    let value: unknown;
    try {
      value = callbacks.parse ? callbacks.parse(edit.draft, row.row) : edit.draft;
    } catch (error) {
      editing.value = Object.freeze({
        ...edit,
        error: error instanceof Error ? error.message : String(error),
      });
      return false;
    }
    const error = callbacks.validate?.(value, row.row) ?? null;
    if (error !== null) {
      editing.value = Object.freeze({ ...edit, error });
      return false;
    }
    editing.value = null;
    options.onCellEdit?.(
      Object.freeze({
        row: row.row,
        rowId: row.id,
        columnId: column.id,
        value,
        previous: cellValue(column, row.row),
      }),
    );
    return true;
  }

  function cancelEdit(): boolean {
    if (!editing.value) return false;
    editing.value = null;
    return true;
  }

  function position(rowIndex: number, columnIndex: number): DataGridCellPosition | null {
    const column = columnModels.value[clamp(columnIndex, 0, columnModels.value.length - 1)];
    if (!column) return null;
    if (rowIndex < 0) return { rowId: null, columnId: column.id };
    const row = rowModels.value[clamp(rowIndex, 0, rowModels.value.length - 1)];
    return row ? { rowId: row.id, columnId: column.id } : { rowId: null, columnId: column.id };
  }

  function handleEditingKey(event: KeyboardEvent): boolean {
    if (event.key === "Escape") return cancelEdit();
    if (event.key === "Enter" || event.key === "Tab") {
      commitEdit();
      return true;
    }
    return false;
  }

  function handleKeydown(event: KeyboardEvent): boolean {
    if (event.isComposing) return false;
    if (editing.value) return handleEditingKey(event);
    const current = activeCell.value ?? position(-1, 0);
    if (!current) return false;
    const columnIndex = Math.max(
      0,
      columnModels.value.findIndex((column) => column.id === current.columnId),
    );
    const rowIndex =
      current.rowId === null ? -1 : rowModels.value.findIndex((row) => row.id === current.rowId);
    const row = current.rowId === null ? undefined : rowModels.value[rowIndex];
    const lastRow = rowModels.value.length - 1;
    const lastColumn = columnModels.value.length - 1;
    const forward = dir.value === "rtl" ? "ArrowLeft" : "ArrowRight";
    const backward = dir.value === "rtl" ? "ArrowRight" : "ArrowLeft";
    const modifier = event.ctrlKey || event.metaKey;
    const pageSize = toValue(options.pageSize) ?? 10;
    let next: DataGridCellPosition | null = null;

    if (
      current.rowId === null &&
      modifier &&
      event.shiftKey &&
      (event.key === forward || event.key === backward)
    ) {
      return moveColumn(current.columnId, columnIndex + (event.key === forward ? 1 : -1));
    }
    if (
      current.rowId === null &&
      event.altKey &&
      (event.key === forward || event.key === backward)
    ) {
      const model = columnModels.value[columnIndex];
      return model
        ? resizeColumn(model.id, model.width + (event.key === forward ? 10 : -10))
        : false;
    }
    if (
      row &&
      columnIndex === 0 &&
      treeGrid.value &&
      (event.key === forward || event.key === backward)
    ) {
      if (event.key === forward && row.expandable && !row.expanded)
        return toggleExpanded(row.id, true);
      if (event.key === backward && row.expanded) return toggleExpanded(row.id, false);
      if (event.key === backward && row.parentId !== null) {
        next = { rowId: row.parentId, columnId: current.columnId };
      }
    }
    if (!next) {
      switch (event.key) {
        case forward:
          next = position(rowIndex, columnIndex + 1);
          break;
        case backward:
          next = position(rowIndex, columnIndex - 1);
          break;
        case "ArrowDown":
          next = position(Math.min(rowIndex + 1, lastRow), columnIndex);
          break;
        case "ArrowUp":
          next = position(Math.max(rowIndex - 1, -1), columnIndex);
          break;
        case "PageDown":
          next = position(Math.min(rowIndex + pageSize, lastRow), columnIndex);
          break;
        case "PageUp":
          next = position(Math.max(rowIndex - pageSize, -1), columnIndex);
          break;
        case "Home":
          next = position(modifier ? -1 : rowIndex, 0);
          break;
        case "End":
          next = position(modifier ? lastRow : rowIndex, lastColumn);
          break;
        case "Enter":
        case "F2":
          if (current.rowId === null) return toggleSort(current.columnId, event.shiftKey);
          return startEdit(current);
        case " ":
          if (current.rowId === null) return toggleSort(current.columnId, event.shiftKey);
          if (selectionMode.value === "none") return false;
          select(current.rowId, event.shiftKey ? "range" : "toggle");
          return true;
        case "a":
        case "A":
          if (!modifier || selectionMode.value !== "multiple") return false;
          setState(
            "selection",
            rowModels.value.map((model) => model.id),
          );
          return true;
        default:
          return false;
      }
    }
    if (!next) return false;
    if (
      event.shiftKey &&
      selectionMode.value === "multiple" &&
      next.rowId !== null &&
      current.rowId !== null &&
      (event.key === "ArrowDown" || event.key === "ArrowUp")
    ) {
      if (anchorRowId === null) anchorRowId = current.rowId;
      select(next.rowId, "range");
    }
    activeCell.value = next;
    return true;
  }

  return {
    state,
    rowModels,
    columnModels,
    totalWidth,
    allSelected,
    someSelected,
    treeGrid,
    selectionMode,
    activeCell,
    editing,
    setState,
    toggleSort,
    setGlobalFilter: (value) => setState("globalFilter", value),
    setColumnFilter,
    setColumnVisible,
    moveColumn,
    pinColumn,
    resizeColumn,
    select,
    toggleAllSelected,
    toggleExpanded,
    setActiveCell,
    startEdit,
    setDraft,
    commitEdit,
    cancelEdit,
    cellValue: (row, column) => cellValue(column, row),
    cellText: (row, column) => formatCell(column, row),
    handleKeydown,
  };
}
