<script setup lang="ts" generic="Row, Column extends DataGridColumn<Row>">
import { computed, nextTick, onMounted, useTemplateRef, watch } from "vue";

import { useDeterministicId } from "../../foundations/id/deterministic-id.ts";
import { useVirtualizer } from "../../interaction/virtualizer/virtualizer.ts";
import DataGridCell from "./data-grid-cell.vue";
import DataGridCellEditor from "./data-grid-cell-editor.vue";
import DataGridColumnHeader from "./data-grid-column-header.vue";
import { isEditable } from "./data-grid-columns.ts";
import { dataGridContext } from "./data-grid-context.ts";
import { useDataGrid } from "./data-grid-model.ts";
import type { DataGridState, DataGridStateKey } from "./data-grid-model.ts";
import DataGridRow from "./data-grid-row.vue";
import type {
  DataGridCellEditEvent,
  DataGridCellPosition,
  DataGridCellSlotProps,
  DataGridColumn,
  DataGridColumnModel,
  DataGridEditorSlotProps,
  DataGridHeaderSlotProps,
  DataGridPinning,
  DataGridRowModel,
  DataGridSelectionMode,
  DataGridSort,
} from "./data-grid-types.ts";

const {
  rows,
  columns,
  id = undefined,
  getRowId = undefined,
  getSubRows = undefined,
  filterRow = undefined,
  selectionMode = "none",
  pageSize = 10,
  dir = "ltr",
  virtualize = false,
  rowHeight = 36,
  overscan = 4,
  initialViewportHeight = 400,
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  defaultState = undefined,
  sorting = undefined,
  globalFilter = undefined,
  columnFilters = undefined,
  columnVisibility = undefined,
  columnOrder = undefined,
  columnPinning = undefined,
  columnSizing = undefined,
  selection = undefined,
  expanded = undefined,
} = defineProps<{
  /**
   * Source rows. Nested rows come from `getSubRows`.
   *
   * @default undefined
   */
  readonly rows: readonly Row[];

  /**
   * Column definitions, usually built with `createColumnHelper<Row>()`.
   *
   * @default undefined
   */
  readonly columns: readonly Column[];

  /**
   * Consumer-owned grid id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Stable row id; defaults to the index path (`"3"`, `"3.1"`).
   *
   * @default undefined
   */
  readonly getRowId?: (row: Row, index: number, parentId: string | null) => string;

  /**
   * Sub rows; providing it renders `role="treegrid"`.
   *
   * @default undefined
   */
  readonly getSubRows?: (row: Row) => readonly Row[] | null | undefined;

  /**
   * Extra row predicate combined with column and global filters.
   *
   * @default undefined
   */
  readonly filterRow?: (row: Row) => boolean;

  /**
   * Row selection policy.
   *
   * @default "none"
   */
  readonly selectionMode?: DataGridSelectionMode;

  /**
   * Rows moved by PageUp and PageDown.
   *
   * @default 10
   */
  readonly pageSize?: number;

  /**
   * Reading direction for horizontal arrow keys.
   *
   * @default "ltr"
   */
  readonly dir?: "ltr" | "rtl";

  /**
   * Render only the rows in view (the root element is the scroll container).
   *
   * @default false
   */
  readonly virtualize?: boolean;

  /**
   * Fixed row height in CSS pixels used by virtualization.
   *
   * @default 36
   */
  readonly rowHeight?: number;

  /**
   * Extra virtualized rows rendered above and below the viewport.
   *
   * @default 4
   */
  readonly overscan?: number;

  /**
   * Viewport height assumed during SSR and before measurement.
   *
   * @default 400
   */
  readonly initialViewportHeight?: number;

  /**
   * Accessible name of the grid.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Ids that label the grid.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Initial uncontrolled state slices.
   *
   * @default undefined
   */
  readonly defaultState?: Partial<DataGridState>;

  /**
   * Controlled sort keys (`v-model:sorting`).
   *
   * @default undefined
   */
  readonly sorting?: readonly DataGridSort[];

  /**
   * Controlled global filter text (`v-model:global-filter`).
   *
   * @default undefined
   */
  readonly globalFilter?: string;

  /**
   * Controlled per-column filter values (`v-model:column-filters`).
   *
   * @default undefined
   */
  readonly columnFilters?: Readonly<Record<string, unknown>>;

  /**
   * Controlled column visibility (`v-model:column-visibility`).
   *
   * @default undefined
   */
  readonly columnVisibility?: Readonly<Record<string, boolean>>;

  /**
   * Controlled column order (`v-model:column-order`).
   *
   * @default undefined
   */
  readonly columnOrder?: readonly string[];

  /**
   * Controlled pinning (`v-model:column-pinning`).
   *
   * @default undefined
   */
  readonly columnPinning?: DataGridPinning;

  /**
   * Controlled widths (`v-model:column-sizing`).
   *
   * @default undefined
   */
  readonly columnSizing?: Readonly<Record<string, number>>;

  /**
   * Controlled selected row ids (`v-model:selection`).
   *
   * @default undefined
   */
  readonly selection?: readonly string[];

  /**
   * Controlled expanded row ids (`v-model:expanded`).
   *
   * @default undefined
   */
  readonly expanded?: readonly string[];
}>();

const emit = defineEmits<{
  /** Fired when sorting changes. */
  "update:sorting": [value: readonly DataGridSort[]];
  /** Fired when the global filter changes. */
  "update:globalFilter": [value: string];
  /** Fired when a column filter changes. */
  "update:columnFilters": [value: Readonly<Record<string, unknown>>];
  /** Fired when column visibility changes. */
  "update:columnVisibility": [value: Readonly<Record<string, boolean>>];
  /** Fired when the column order changes. */
  "update:columnOrder": [value: readonly string[]];
  /** Fired when pinning changes. */
  "update:columnPinning": [value: DataGridPinning];
  /** Fired when a column is resized. */
  "update:columnSizing": [value: Readonly<Record<string, number>>];
  /** Fired when the row selection changes. */
  "update:selection": [value: readonly string[]];
  /** Fired when rows expand or collapse. */
  "update:expanded": [value: readonly string[]];
  /** Fired when a cell edit commits; update `rows` to apply it. */
  "cell-edit": [event: DataGridCellEditEvent<Row>];
}>();

defineSlots<{
  /** Column header contents. Defaults to the column's `header` or id. */
  header?(props: DataGridHeaderSlotProps<Row, Column>): unknown;
  /** Cell contents, discriminated by `columnId`. Defaults to the formatted value. */
  cell?(props: DataGridCellSlotProps<Row, Column>): unknown;
  /** Editor for the cell being edited. Defaults to a text input. */
  editor?(props: DataGridEditorSlotProps<Row>): unknown;
  /** Rendered in the body when no rows remain after filtering. */
  empty?(): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");
const gridId = useDeterministicId({ id: () => id, hint: "data-grid" });

function emitState<Key extends DataGridStateKey>(key: Key, value: DataGridState[Key]): void {
  const payload: DataGridState = { ...grid.state.value, [key]: value };
  switch (key) {
    case "sorting":
      emit("update:sorting", payload.sorting);
      break;
    case "globalFilter":
      emit("update:globalFilter", payload.globalFilter);
      break;
    case "columnFilters":
      emit("update:columnFilters", payload.columnFilters);
      break;
    case "columnVisibility":
      emit("update:columnVisibility", payload.columnVisibility);
      break;
    case "columnOrder":
      emit("update:columnOrder", payload.columnOrder);
      break;
    case "columnPinning":
      emit("update:columnPinning", payload.columnPinning);
      break;
    case "columnSizing":
      emit("update:columnSizing", payload.columnSizing);
      break;
    case "selection":
      emit("update:selection", payload.selection);
      break;
    default:
      emit("update:expanded", payload.expanded);
  }
}

const grid = useDataGrid<Row, Column>({
  rows: () => rows,
  columns: () => columns,
  getRowId: (row, index, parentId) =>
    getRowId
      ? getRowId(row, index, parentId)
      : parentId === null
        ? String(index)
        : `${parentId}.${index}`,
  getSubRows: (row: Row) => getSubRows?.(row),
  treeGrid: () => getSubRows !== undefined,
  filterRow: () => filterRow,
  selectionMode: () => selectionMode,
  pageSize: () => pageSize,
  dir: () => dir,
  state: {
    sorting: () => sorting,
    globalFilter: () => globalFilter,
    columnFilters: () => columnFilters,
    columnVisibility: () => columnVisibility,
    columnOrder: () => columnOrder,
    columnPinning: () => columnPinning,
    columnSizing: () => columnSizing,
    selection: () => selection,
    expanded: () => expanded,
  },
  defaultState: () => defaultState,
  onStateChange: emitState,
  onCellEdit: (event) => emit("cell-edit", event),
});

function initialRect(): { readonly width: number; readonly height: number } {
  return { width: 0, height: initialViewportHeight };
}

const virtualizer = useVirtualizer({
  count: () => grid.rowModels.value.length,
  itemSize: () => rowHeight,
  overscan: () => overscan,
  initialRect: initialRect(),
  getItemKey: (index) => grid.rowModels.value[index]?.id ?? index,
});

interface RenderedRow {
  readonly model: DataGridRowModel<Row>;
  readonly start: number | null;
}

const renderedRows = computed<readonly RenderedRow[]>(() => {
  const models = grid.rowModels.value;
  if (!virtualize) return models.map((model) => ({ model, start: null }));
  return virtualizer.virtualItems.value.flatMap((item) => {
    const model = models[item.index];
    return model ? [{ model, start: item.start }] : [];
  });
});

onMounted(() => {
  if (virtualize) virtualizer.setViewport(element.value);
});

function cellElement(position: DataGridCellPosition): HTMLElement | null {
  const selector = position.rowId === null ? '[role="columnheader"]' : '[role="gridcell"]';
  for (const candidate of element.value?.querySelectorAll<HTMLElement>(selector) ?? []) {
    if (
      candidate.dataset.columnId === position.columnId &&
      (position.rowId === null || candidate.dataset.rowId === position.rowId)
    ) {
      return candidate;
    }
  }
  return null;
}

async function focusActiveCell(): Promise<void> {
  const position = grid.activeCell.value;
  if (!position) return;
  if (virtualize && position.rowId !== null) {
    const index = grid.rowModels.value.findIndex((model) => model.id === position.rowId);
    if (index >= 0) virtualizer.scrollToIndex(index, "auto");
  }
  await nextTick();
  cellElement(position)?.focus({ preventScroll: !virtualize });
}

function focusCell(position: DataGridCellPosition): void {
  grid.setActiveCell(position);
  void focusActiveCell();
}

dataGridContext.provide({
  id: gridId,
  selectionMode: grid.selectionMode,
  treeGrid: grid.treeGrid,
  activeCell: grid.activeCell,
  editing: grid.editing,
  focusCell,
  rowClick: onRowClick,
  select: (rowId, intent) => grid.select(rowId, intent),
  toggleExpanded: (rowId) => grid.toggleExpanded(rowId),
  toggleSort: (columnId, multi) => grid.toggleSort(columnId, multi),
  resizeColumn: (columnId, width) => grid.resizeColumn(columnId, width),
  startEdit: (position) => grid.startEdit(position),
  setDraft: (draft) => grid.setDraft(draft),
  commitEdit: () => grid.commitEdit(),
  cancelEdit: () => grid.cancelEdit(),
});

// Leaving edit mode returns focus to the edited cell.
watch(
  () => grid.editing.value === null,
  (idle, wasIdle) => {
    if (idle && !wasIdle) void focusActiveCell();
  },
);

function onKeydown(event: KeyboardEvent): void {
  if (event.defaultPrevented) return;
  const wasEditing = grid.editing.value !== null;
  if (!grid.handleKeydown(event)) return;
  event.preventDefault();
  // Entering edit mode hands focus to the editor; leaving it is handled by the watcher.
  if (!wasEditing && grid.editing.value === null) void focusActiveCell();
}

function onFocusin(event: FocusEvent): void {
  const target = event.target;
  if (!(target instanceof HTMLElement) || grid.editing.value) return;
  const columnId = target.dataset.columnId;
  if (columnId === undefined) return;
  const role = target.getAttribute("role");
  if (role === "columnheader") grid.setActiveCell({ rowId: null, columnId });
  else if (role === "gridcell" && target.dataset.rowId !== undefined) {
    grid.setActiveCell({ rowId: target.dataset.rowId, columnId });
  }
}

function onRowClick(event: MouseEvent, rowId: string): void {
  if (grid.selectionMode.value === "none" || grid.editing.value) return;
  const intent = event.shiftKey ? "range" : event.ctrlKey || event.metaKey ? "toggle" : "replace";
  grid.select(rowId, grid.selectionMode.value === "single" ? "replace" : intent);
}

function headerSlotProps(
  model: DataGridColumnModel<Row, Column>,
): DataGridHeaderSlotProps<Row, Column> {
  return {
    column: model.column,
    columnModel: model,
    allSelected: grid.allSelected.value,
    someSelected: grid.someSelected.value,
    toggleAllSelected: () => void grid.toggleAllSelected(),
  };
}

function cellSlotProps(
  row: DataGridRowModel<Row>,
  model: DataGridColumnModel<Row, Column>,
): DataGridCellSlotProps<Row, Column> {
  const active =
    grid.activeCell.value?.rowId === row.id && grid.activeCell.value.columnId === model.id;
  return cellProps(model.column, row, active);
}

// The discriminated slot union is built per column; `column` and `value`
// always come from the same definition, so the union member is exact.
function cellProps(
  column: Column,
  row: DataGridRowModel<Row>,
  active: boolean,
): DataGridCellSlotProps<Row, Column> {
  const props = {
    columnId: column.id,
    column,
    row: row.row,
    rowModel: row,
    value: grid.cellValue(row.row, column),
    text: grid.cellText(row.row, column),
    active,
    toggleSelected: () => void grid.select(row.id, "toggle"),
    toggleExpanded: () => void grid.toggleExpanded(row.id),
  };
  return props as DataGridCellSlotProps<Row, Column>;
}

function editorSlotProps(row: DataGridRowModel<Row>): DataGridEditorSlotProps<Row> {
  const edit = grid.editing.value;
  return {
    row: row.row,
    columnId: edit?.columnId ?? "",
    draft: edit?.draft ?? "",
    error: edit?.error ?? null,
    setDraft: (value) => grid.setDraft(value),
    commit: () => grid.commitEdit(),
    cancel: () => void grid.cancelEdit(),
  };
}

function isEditing(row: DataGridRowModel<Row>, model: DataGridColumnModel<Row, Column>): boolean {
  return grid.editing.value?.rowId === row.id && grid.editing.value.columnId === model.id;
}

const rootProps = computed(() => ({
  role: grid.treeGrid.value ? "treegrid" : "grid",
  style: { "--vize-ui-data-grid-total-width": `${grid.totalWidth.value}px` },
  onFocusin,
  onKeydown,
}));
const bodyProps = computed(() =>
  virtualize
    ? { style: { "--vize-ui-data-grid-body-height": `${virtualizer.totalSize.value}px` } }
    : {},
);

defineExpose({
  element,
  controller: grid,
  focusCell,
  scrollToRow: (rowId: string) => {
    const index = grid.rowModels.value.findIndex((model) => model.id === rowId);
    if (index >= 0) virtualizer.scrollToIndex(index, "auto");
  },
});
</script>

<template>
  <div
    v-bind="rootProps"
    :id="gridId"
    ref="element"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-rowcount="grid.rowModels.value.length + 1"
    :aria-colcount="grid.columnModels.value.length"
    :aria-multiselectable="grid.selectionMode.value === 'multiple' ? 'true' : undefined"
    :dir
    data-vize-ui="data-grid"
    part="root"
    :data-virtualized="virtualize ? 'true' : undefined"
    :data-empty="grid.rowModels.value.length === 0 ? 'true' : undefined"
  >
    <div role="rowgroup" data-vize-ui="data-grid-header" part="header">
      <div role="row" aria-rowindex="1" data-vize-ui="data-grid-header-row" part="header-row">
        <DataGridColumnHeader
          v-for="model in grid.columnModels.value"
          :key="model.id"
          :column-id="model.id"
          :col-index="model.index + 1"
          :label="model.column.header ?? model.id"
          :width="model.width"
          :min-width="model.column.minWidth ?? 40"
          :max-width="model.column.maxWidth ?? Number.POSITIVE_INFINITY"
          :pin="model.pin"
          :pin-offset="model.pinOffset"
          :align="model.column.align ?? 'start'"
          :sortable="model.sortable"
          :sort-direction="model.sortDirection"
          :sort-index="model.sortIndex"
          :resizable="model.column.resizable !== false"
        >
          <slot name="header" v-bind="headerSlotProps(model)">{{
            model.column.header ?? model.id
          }}</slot>
        </DataGridColumnHeader>
      </div>
    </div>
    <div v-bind="bodyProps" role="rowgroup" data-vize-ui="data-grid-body" part="body">
      <DataGridRow
        v-for="entry in renderedRows"
        :key="entry.model.id"
        :row-id="entry.model.id"
        :row-index="entry.model.index + 2"
        :selected="entry.model.selected"
        :depth="entry.model.depth"
        :expandable="entry.model.expandable"
        :expanded="entry.model.expanded"
        :start="entry.start"
      >
        <DataGridCell
          v-for="model in grid.columnModels.value"
          :key="model.id"
          :row-id="entry.model.id"
          :column-id="model.id"
          :col-index="model.index + 1"
          :width="model.width"
          :pin="model.pin"
          :pin-offset="model.pinOffset"
          :align="model.column.align ?? 'start'"
          :editable="isEditable(model.column, entry.model.row)"
        >
          <template v-if="isEditing(entry.model, model)">
            <slot name="editor" v-bind="editorSlotProps(entry.model)">
              <DataGridCellEditor :label="model.column.header ?? model.id" />
            </slot>
          </template>
          <slot v-else name="cell" v-bind="cellSlotProps(entry.model, model)">{{
            grid.cellText(entry.model.row, model.column)
          }}</slot>
        </DataGridCell>
      </DataGridRow>
      <div v-if="grid.rowModels.value.length === 0" data-vize-ui="data-grid-empty" part="empty">
        <slot name="empty" />
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>
