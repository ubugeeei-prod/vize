/** One table row addressable by string column keys. */
export type TableRow = Readonly<Record<string, unknown>>;

export type TableAlignment = "left" | "center" | "right";
export type TableSortDirection = "ascending" | "descending";

/** Typed table column definition. */
export interface TableColumn<Row extends TableRow = TableRow> {
  /** Row key read by this column. */
  key: Extract<keyof Row, string> | string;
  /** Column header text. */
  header: string;
  /** Preferred terminal-cell width. */
  width?: number | string;
  /** Smallest width accepted by keyboard resizing. */
  minWidth?: number;
  /** Largest width accepted by keyboard resizing. */
  maxWidth?: number;
  /** Cell alignment within a numeric width. */
  align?: TableAlignment;
  /** Allow header activation to own sorting. */
  sortable?: boolean;
  /** Optional value formatter. */
  format?: (value: unknown, row: Row, rowIndex: number) => string;
  /** Optional comparison hook used when this column owns sorting. */
  compare?: (left: Row, right: Row) => number;
}

export interface TableViewportRange {
  readonly visibleStart: number;
  readonly visibleEnd: number;
  readonly retainedStart: number;
  readonly retainedEnd: number;
}

/** Public table behavior, viewport, and appearance contract. */
export interface TableProps<Row extends TableRow = TableRow> {
  columns: readonly TableColumn<Row>[];
  data: readonly Row[];
  showHeader?: boolean;
  border?: "none" | "single" | "double" | "rounded";
  headerFg?: string;
  headerBg?: string;
  rowFg?: string;
  stripedBg?: string;
  cellPadding?: number;
  /** Controlled sort column key. */
  sortKey?: string;
  /** Controlled sort direction. */
  sortDirection?: TableSortDirection;
  /** Controlled first visible sorted row. */
  offset?: number;
  /** Visible row count; omit to render every row. */
  viewportSize?: number;
  /** Rows retained outside each viewport edge. */
  overscan?: number;
  /** Whether the header owns keyboard navigation, sort, resize, and scrolling. */
  focused?: boolean;
  /** Controlled focused column index. */
  focusedColumn?: number;
  /** Controlled width overrides keyed by column key. */
  columnWidths?: Readonly<Record<string, number>>;
  /** Keyboard resize delta in terminal cells. */
  resizeStep?: number;
}

export interface TableCellSlotProps<Row extends TableRow = TableRow> {
  readonly row: Row;
  readonly rowIndex: number;
  readonly column: TableColumn<Row>;
  readonly value: unknown;
  readonly content: string;
}
