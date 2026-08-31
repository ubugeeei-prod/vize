import type {
  TableBodySlotState,
  TableCaptionSlotState,
  TableCellSlotState,
  TableHeaderSlotState,
  TableHeadSlotState,
  TableRowSlotState,
  TableSlotState,
} from "./table-types.ts";

/** Slots exposed by the root Table component. */
export interface TableSlots {
  /** Renders caption, section, row, and cell children with table hook state. */
  default(props: TableSlotState): unknown;
}

/** Slots exposed by the caption component. */
export interface TableCaptionSlots {
  /** Renders the native caption content. */
  default(props: TableCaptionSlotState): unknown;
}

/** Slots exposed by the head section component. */
export interface TableHeadSlots {
  /** Renders one or more native table rows. */
  default(props: TableHeadSlotState): unknown;
}

/** Slots exposed by the body section component. */
export interface TableBodySlots {
  /** Renders one or more native table rows. */
  default(props: TableBodySlotState): unknown;
}

/** Slots exposed by the row component. */
export interface TableRowSlots {
  /** Renders native header and data cells. */
  default(props: TableRowSlotState): unknown;
}

/** Slots exposed by the header cell component. */
export interface TableHeaderSlots {
  /** Renders native header cell content. */
  default(props: TableHeaderSlotState): unknown;
}

/** Slots exposed by the data cell component. */
export interface TableCellSlots {
  /** Renders native data cell content. */
  default(props: TableCellSlotState): unknown;
}

/** Public component instance state exposed by the root Table component. */
export interface TableExpose extends TableSlotState {
  /** Rendered native table element. */
  readonly element: HTMLTableElement | null;
}

/** Public component instance state exposed by the caption component. */
export interface TableCaptionExpose extends TableCaptionSlotState {
  /** Rendered native caption element. */
  readonly element: HTMLTableCaptionElement | null;
}

/** Public component instance state exposed by the head section component. */
export interface TableHeadExpose extends TableHeadSlotState {
  /** Rendered native table head element. */
  readonly element: HTMLTableSectionElement | null;
}

/** Public component instance state exposed by the body section component. */
export interface TableBodyExpose extends TableBodySlotState {
  /** Rendered native table body element. */
  readonly element: HTMLTableSectionElement | null;
}

/** Public component instance state exposed by the row component. */
export interface TableRowExpose extends TableRowSlotState {
  /** Rendered native table row element. */
  readonly element: HTMLTableRowElement | null;
}

/** Public component instance state exposed by the header cell component. */
export interface TableHeaderExpose extends TableHeaderSlotState {
  /** Rendered native table header cell element. */
  readonly element: HTMLTableCellElement | null;
}

/** Public component instance state exposed by the data cell component. */
export interface TableCellExpose extends TableCellSlotState {
  /** Rendered native table data cell element. */
  readonly element: HTMLTableCellElement | null;
}
