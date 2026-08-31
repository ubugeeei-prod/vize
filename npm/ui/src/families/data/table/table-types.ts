import type { CSSProperties } from "vue";

/** Native table layout modes mirrored by {@link Table}. */
export type TableLayout = "auto" | "fixed";

/** Consumer density hooks mirrored by {@link Table} through `data-density`. */
export type TableDensity = "compact" | "normal" | "spacious";

/** Native caption placement hooks mirrored by {@link TableCaption}. */
export type TableCaptionSide = "top" | "bottom";

/** Consumer-owned row state hooks mirrored by {@link TableRow}. */
export type TableRowState = "default" | "selected";

/** Logical text alignment hooks mirrored by {@link TableHeader} and {@link TableCell}. */
export type TableCellAlign = "start" | "center" | "end";

/** Native `scope` values accepted by table header cells. */
export type TableHeaderScope = "col" | "colgroup" | "row" | "rowgroup";

/** Stable part names emitted by the semantic table family. */
export type TablePart = "body" | "caption" | "cell" | "head" | "header" | "root" | "row";

/** Stable `data-vize-ui` values emitted by the semantic table family. */
export type TableDataName =
  | "table"
  | "table-body"
  | "table-caption"
  | "table-cell"
  | "table-head"
  | "table-header"
  | "table-row";

/** Stable data attributes emitted by one or more semantic table components. */
export type TableDataAttribute =
  | "data-align"
  | "data-caption-side"
  | "data-colspan"
  | "data-density"
  | "data-layout"
  | "data-rowspan"
  | "data-scope"
  | "data-section"
  | "data-selected"
  | "data-state"
  | "data-vize-ui";

/** CSS custom properties authored inline by semantic table components. */
export type TableCssCustomProperty =
  | "--vize-ui-table-caption-side"
  | "--vize-ui-table-cell-align"
  | "--vize-ui-table-layout";

/** Inline style contract applied to the root native table. */
export interface TableStyle extends Readonly<CSSProperties> {
  /** Consumer-overridable value read by `table-layout`. */
  readonly "--vize-ui-table-layout": TableLayout;

  /** Native table layout declaration. */
  readonly tableLayout: TableLayout;
}

/** Inline style contract applied to a native caption. */
export interface TableCaptionStyle extends Readonly<CSSProperties> {
  /** Consumer-overridable value read by `caption-side`. */
  readonly "--vize-ui-table-caption-side": TableCaptionSide;

  /** Native caption placement declaration. */
  readonly captionSide: TableCaptionSide;
}

/** Inline style contract applied to native header and data cells. */
export interface TableCellStyle extends Readonly<CSSProperties> {
  /** Consumer-overridable value read by `text-align`. */
  readonly "--vize-ui-table-cell-align": TableCellAlign;

  /** Native logical text alignment declaration. */
  readonly textAlign: TableCellAlign;
}

/** Public props accepted by the root Table component. */
export interface TableProps {
  /**
   * Native CSS `table-layout` mode.
   *
   * @default "auto"
   */
  readonly layout?: TableLayout;

  /**
   * Consumer density token mirrored to `data-density`; no spacing CSS is emitted.
   *
   * @default "normal"
   */
  readonly density?: TableDensity;
}

/** Props accepted by the caption component. */
export interface TableCaptionProps {
  /**
   * Native CSS `caption-side` placement.
   *
   * @default "top"
   */
  readonly side?: TableCaptionSide;
}

/** Props accepted by the head section component. */
export type TableHeadProps = Record<never, never>;

/** Props accepted by the body section component. */
export type TableBodyProps = Record<never, never>;

/** Props accepted by the row component. */
export interface TableRowProps {
  /**
   * Consumer-owned row state mirrored to data attributes.
   *
   * @default "default"
   */
  readonly state?: TableRowState;
}

/** Props accepted by the header cell component. */
export interface TableHeaderProps {
  /**
   * Native header scope for assistive technology.
   *
   * @default "col"
   */
  readonly scope?: TableHeaderScope;

  /**
   * Native abbreviated header text for compact assistive output.
   *
   * @default undefined
   */
  readonly abbr?: string;

  /**
   * Native column span.
   *
   * @default undefined
   */
  readonly colspan?: number;

  /**
   * Native row span.
   *
   * @default undefined
   */
  readonly rowspan?: number;

  /**
   * Logical text alignment mirrored to `data-align` and `--vize-ui-table-cell-align`.
   *
   * @default "start"
   */
  readonly align?: TableCellAlign;
}

/** Props accepted by the data cell component. */
export interface TableCellProps {
  /**
   * Space-separated ids of native header cells that describe this data cell.
   *
   * @default undefined
   */
  readonly headers?: string;

  /**
   * Native column span.
   *
   * @default undefined
   */
  readonly colspan?: number;

  /**
   * Native row span.
   *
   * @default undefined
   */
  readonly rowspan?: number;

  /**
   * Logical text alignment mirrored to `data-align` and `--vize-ui-table-cell-align`.
   *
   * @default "start"
   */
  readonly align?: TableCellAlign;
}

/** No custom events are emitted by the root Table component. */
export type TableEmits = Record<never, never>;

/** No custom events are emitted by the caption component. */
export type TableCaptionEmits = Record<never, never>;

/** No custom events are emitted by the head section component. */
export type TableHeadEmits = Record<never, never>;

/** No custom events are emitted by the body section component. */
export type TableBodyEmits = Record<never, never>;

/** No custom events are emitted by the row component. */
export type TableRowEmits = Record<never, never>;

/** No custom events are emitted by the header cell component. */
export type TableHeaderEmits = Record<never, never>;

/** No custom events are emitted by the data cell component. */
export type TableCellEmits = Record<never, never>;

/** State exposed to the root Table default slot. */
export interface TableSlotState {
  /** Native CSS `table-layout` mode. */
  readonly layout: TableLayout;

  /** Consumer density token. */
  readonly density: TableDensity;

  /** Inline native table style object applied to the host. */
  readonly style: TableStyle;
}

/** State exposed to the caption default slot. */
export interface TableCaptionSlotState {
  /** Native CSS `caption-side` placement. */
  readonly side: TableCaptionSide;

  /** Inline caption style object applied to the host. */
  readonly style: TableCaptionStyle;
}

/** State exposed to the head section default slot. */
export interface TableHeadSlotState {
  /** Stable semantic section token. */
  readonly section: "head";
}

/** State exposed to the body section default slot. */
export interface TableBodySlotState {
  /** Stable semantic section token. */
  readonly section: "body";
}

/** State exposed to the row default slot. */
export interface TableRowSlotState {
  /** Consumer-owned row state hook. */
  readonly state: TableRowState;

  /** Whether this row is marked selected for consumer styling. */
  readonly selected: boolean;
}

/** State exposed to the header cell default slot. */
export interface TableHeaderSlotState {
  /** Native header scope. */
  readonly scope: TableHeaderScope;

  /** Native abbreviated header text. */
  readonly abbr: string | undefined;

  /** Native column span. */
  readonly colspan: number | undefined;

  /** Native row span. */
  readonly rowspan: number | undefined;

  /** Logical text alignment hook. */
  readonly align: TableCellAlign;

  /** Inline cell style object applied to the host. */
  readonly style: TableCellStyle;
}

/** State exposed to the data cell default slot. */
export interface TableCellSlotState {
  /** Native header id references. */
  readonly headers: string | undefined;

  /** Native column span. */
  readonly colspan: number | undefined;

  /** Native row span. */
  readonly rowspan: number | undefined;

  /** Logical text alignment hook. */
  readonly align: TableCellAlign;

  /** Inline cell style object applied to the host. */
  readonly style: TableCellStyle;
}
