/** One table row addressable by string column keys. */
export type TableRow = Readonly<Record<string, unknown>>;

/** Horizontal alignment of content within a terminal table column. */
export type TableAlignment = "left" | "center" | "right";
/** Ascending or descending ordering requested for a table column. */
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

/** Visible and retained row-index bounds of a table viewport. */
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
  /** Render the table header.
   * @default true
   */
  showHeader?: boolean;
  /** Table border style.
   * @default "single"
   */
  border?: "none" | "single" | "double" | "rounded";
  /** Default foreground color for header cells.
   * @default "white"
   */
  headerFg?: string;
  /** Default background color for header cells.
   * @default undefined
   */
  headerBg?: string;
  /** Default foreground color for body cells.
   * @default undefined
   */
  rowFg?: string;
  /** Background color used for alternating body rows.
   * @default undefined
   */
  stripedBg?: string;
  /** Gap between adjacent table cells, in terminal cells.
   * @default 1
   */
  cellPadding?: number;
  /** Controlled sort column key.
   * @default undefined
   */
  sortKey?: string;
  /** Controlled sort direction.
   * @default "ascending"
   */
  sortDirection?: TableSortDirection;
  /** Controlled first visible sorted row.
   * @default 0
   */
  offset?: number;
  /** Visible row count; omit to render every row.
   * @default undefined
   */
  viewportSize?: number;
  /** Rows retained outside each viewport edge.
   * @default 2
   */
  overscan?: number;
  /** Whether the header owns keyboard navigation, sort, resize, and scrolling.
   * @default false
   */
  focused?: boolean;
  /** Controlled focused column index.
   * @default undefined
   */
  focusedColumn?: number;
  /** Controlled width overrides keyed by column key.
   * @default undefined
   */
  columnWidths?: Readonly<Record<string, number>>;
  /** Keyboard resize delta in terminal cells.
   * @default 1
   */
  resizeStep?: number;
}

/** Row, column, value, and rendered text passed to a table cell slot. */
export interface TableCellSlotProps<Row extends TableRow = TableRow> {
  readonly row: Row;
  readonly rowIndex: number;
  readonly column: TableColumn<Row>;
  readonly value: unknown;
  readonly content: string;
}
