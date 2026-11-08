/** Table Component - sortable, resizable, bounded tabular data. */

import { computed, defineComponent, h, ref, type PropType, type VNode, watch } from "vue";
import { useInput } from "../composables/useInput.js";
import {
  fitTableCell,
  sortTableRows,
  stringifyTableCell,
  tableColumnWidth,
  tableViewportRange,
} from "./table-utils.js";
import type {
  TableCellSlotProps,
  TableColumn,
  TableProps,
  TableRow,
  TableSortDirection,
} from "./table-types.js";

export type {
  TableAlignment,
  TableCellSlotProps,
  TableColumn,
  TableProps,
  TableRow,
  TableSortDirection,
  TableViewportRange,
} from "./table-types.js";

export const Table = defineComponent({
  name: "Table",
  props: {
    columns: { type: Array as PropType<readonly TableColumn[]>, required: true },
    data: { type: Array as PropType<readonly TableRow[]>, required: true },
    showHeader: { type: Boolean, default: true },
    border: { type: String as PropType<TableProps["border"]>, default: "single" },
    headerFg: { type: String, default: "white" },
    headerBg: String,
    rowFg: String,
    stripedBg: String,
    cellPadding: { type: Number, default: 1 },
    sortKey: String,
    sortDirection: {
      type: String as PropType<TableSortDirection>,
      default: "ascending",
    },
    offset: { type: Number, default: 0 },
    viewportSize: Number,
    overscan: { type: Number, default: 2 },
    focused: { type: Boolean, default: false },
    focusedColumn: Number,
    columnWidths: Object as PropType<Readonly<Record<string, number>>>,
    resizeStep: { type: Number, default: 1 },
  },
  emits: [
    "update:sortKey",
    "update:sortDirection",
    "update:offset",
    "update:focusedColumn",
    "update:columnWidths",
    "sortChange",
    "resize",
  ],
  setup(props, { emit, slots }) {
    const activeColumn = ref(0);
    watch(
      [() => props.focusedColumn, () => props.columns.length],
      ([requested, count]) => {
        activeColumn.value = Math.min(
          Math.max(0, requested ?? activeColumn.value),
          Math.max(0, count - 1),
        );
      },
      { immediate: true },
    );

    const sortedRows = computed(() =>
      sortTableRows(props.data, props.columns, props.sortKey, props.sortDirection),
    );
    const viewport = computed(() =>
      tableViewportRange(sortedRows.value.length, props.offset, props.viewportSize, props.overscan),
    );
    const retainedRows = computed(() =>
      sortedRows.value.slice(viewport.value.retainedStart, viewport.value.retainedEnd),
    );

    const setActiveColumn = (index: number) => {
      const next = Math.min(Math.max(0, index), Math.max(0, props.columns.length - 1));
      if (next === activeColumn.value) return;
      activeColumn.value = next;
      emit("update:focusedColumn", next);
    };
    const activateSort = () => {
      const column = props.columns[activeColumn.value];
      if (!column?.sortable) return;
      const key = String(column.key);
      const direction: TableSortDirection =
        props.sortKey === key && props.sortDirection === "ascending" ? "descending" : "ascending";
      emit("update:sortKey", key);
      emit("update:sortDirection", direction);
      emit("sortChange", key, direction);
    };
    const resizeColumn = (delta: number) => {
      const column = props.columns[activeColumn.value];
      if (!column) return;
      const current = tableColumnWidth(column, props.columnWidths) ?? 10;
      const requested = current + delta;
      const next = tableColumnWidth({ ...column, width: requested }, undefined) ?? current;
      if (next === current) return;
      const widths = { ...props.columnWidths, [String(column.key)]: next };
      emit("update:columnWidths", widths);
      emit("resize", String(column.key), next);
    };
    const scrollBy = (delta: number) => {
      const size = props.viewportSize ?? sortedRows.value.length;
      const maxOffset = Math.max(0, sortedRows.value.length - Math.max(0, Math.floor(size)));
      const next = Math.min(maxOffset, Math.max(0, Math.floor(props.offset) + delta));
      if (next !== props.offset) emit("update:offset", next);
    };

    useInput({
      isActive: computed(() => props.focused),
      onKey: (key, modifiers) => {
        if (key === "left") {
          if (modifiers.shift) resizeColumn(-Math.max(1, Math.floor(props.resizeStep)));
          else setActiveColumn(activeColumn.value - 1);
        } else if (key === "right") {
          if (modifiers.shift) resizeColumn(Math.max(1, Math.floor(props.resizeStep)));
          else setActiveColumn(activeColumn.value + 1);
        } else if (key === "home") setActiveColumn(0);
        else if (key === "end") setActiveColumn(props.columns.length - 1);
        else if (key === "up") scrollBy(-1);
        else if (key === "down") scrollBy(1);
        else if (key === "pageup") scrollBy(-Math.max(1, (props.viewportSize ?? 1) - 1));
        else if (key === "pagedown") scrollBy(Math.max(1, (props.viewportSize ?? 1) - 1));
      },
      onSubmit: activateSort,
    });

    const renderCell = (
      row: TableRow,
      sourceIndex: number,
      column: TableColumn,
      rowIndex: number,
    ): VNode => {
      const raw = row[String(column.key)];
      const formatted = column.format?.(raw, row, sourceIndex) ?? stringifyTableCell(raw);
      const content = fitTableCell(
        formatted,
        tableColumnWidth(column, props.columnWidths),
        column.align,
      );
      const slotProps: TableCellSlotProps = {
        row,
        rowIndex: sourceIndex,
        column,
        value: raw,
        content,
      };
      const cellProps = {
        key: String(column.key),
        fg: props.rowFg,
        bg: rowIndex % 2 === 1 ? props.stripedBg : undefined,
      };
      const slot = slots.cell?.(slotProps);
      return slot ? h("box", cellProps, slot) : h("text", { ...cellProps, text: content });
    };

    return () => {
      const rows: VNode[] = [];
      if (props.showHeader) {
        rows.push(
          h(
            "box",
            { key: "header", style: { flexDirection: "row", gap: props.cellPadding } },
            props.columns.map((column, index) => {
              const indicator =
                props.sortKey === String(column.key)
                  ? props.sortDirection === "descending"
                    ? " ▼"
                    : " ▲"
                  : column.sortable
                    ? " ↕"
                    : "";
              return h("text", {
                key: String(column.key),
                text: fitTableCell(
                  `${column.header}${indicator}`,
                  tableColumnWidth(column, props.columnWidths),
                  column.align,
                ),
                bold: props.focused && index === activeColumn.value,
                fg: props.headerFg,
                bg: props.headerBg,
              });
            }),
          ),
        );
      }

      retainedRows.value.forEach(({ row, sourceIndex }, retainedIndex) => {
        const sortedIndex = viewport.value.retainedStart + retainedIndex;
        const visible =
          sortedIndex >= viewport.value.visibleStart && sortedIndex < viewport.value.visibleEnd;
        rows.push(
          h(
            "box",
            {
              key: `row-${sourceIndex}`,
              style: {
                display: visible ? "flex" : "none",
                flexDirection: "row",
                gap: props.cellPadding,
              },
              "aria-hidden": !visible,
              "data-table-index": sortedIndex,
            },
            props.columns.map((column) => renderCell(row, sourceIndex, column, sortedIndex)),
          ),
        );
      });

      return h(
        "box",
        {
          border: props.border === "none" ? undefined : props.border,
          style: { flexDirection: "column", overflow: "hidden" },
          "aria-role": "table",
          "aria-colcount": props.columns.length,
          "aria-rowcount": sortedRows.value.length,
        },
        rows,
      );
    };
  },
});
