/** Native semantic table component family for accessible tabular data composition. */
export { default as Table } from "./table.vue";
export { default as TableBody } from "./table-body.vue";
export { default as TableCaption } from "./table-caption.vue";
export { default as TableCell } from "./table-cell.vue";
export { default as TableHead } from "./table-head.vue";
export { default as TableHeader } from "./table-header.vue";
export { default as TableRow } from "./table-row.vue";

export type {
  TableBodyExpose,
  TableBodySlots,
  TableCaptionExpose,
  TableCaptionSlots,
  TableCellExpose,
  TableCellSlots,
  TableExpose,
  TableHeaderExpose,
  TableHeaderSlots,
  TableHeadExpose,
  TableHeadSlots,
  TableRowExpose,
  TableRowSlots,
  TableSlots,
} from "./table-contracts.ts";
export type {
  TableBodyEmits,
  TableBodyProps,
  TableBodySlotState,
  TableCaptionEmits,
  TableCaptionProps,
  TableCaptionSide,
  TableCaptionSlotState,
  TableCaptionStyle,
  TableCellAlign,
  TableCellEmits,
  TableCellProps,
  TableCellSlotState,
  TableCellStyle,
  TableCssCustomProperty,
  TableDataAttribute,
  TableDataName,
  TableDensity,
  TableEmits,
  TableHeadEmits,
  TableHeaderEmits,
  TableHeaderProps,
  TableHeaderScope,
  TableHeaderSlotState,
  TableHeadProps,
  TableHeadSlotState,
  TableLayout,
  TablePart,
  TableProps,
  TableRowEmits,
  TableRowProps,
  TableRowSlotState,
  TableRowState,
  TableSlotState,
  TableStyle,
} from "./table-types.ts";
