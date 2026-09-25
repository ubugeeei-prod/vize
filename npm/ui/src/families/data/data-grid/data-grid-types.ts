/** Leaf values that column paths never descend into. */
export type DataGridLeaf =
  | bigint
  | boolean
  | Date
  | null
  | number
  | string
  | symbol
  | undefined
  | readonly unknown[]
  | ((...args: never[]) => unknown);

type PathDepth = readonly unknown[];

/**
 * Dot-separated accessor paths into `Row`, up to three levels deep.
 *
 * `DataGridPath<{ user: { name: string } }>` is `"user" | "user.name"`.
 */
export type DataGridPath<Row, Depth extends PathDepth = []> = Depth["length"] extends 3
  ? never
  : Row extends DataGridLeaf
    ? never
    : {
        [Key in keyof Row & string]:
          | Key
          | (NonNullable<Row[Key]> extends DataGridLeaf
              ? never
              : `${Key}.${DataGridPath<NonNullable<Row[Key]>, [...Depth, unknown]>}`);
      }[keyof Row & string];

/**
 * Value found at an accessor path. Optional or nullable segments make the
 * result `undefined`-able, mirroring the runtime short-circuit.
 */
export type DataGridPathValue<Row, Path extends string> = Path extends `${infer Head}.${infer Rest}`
  ? Head extends keyof Row
    ?
        | DataGridPathValue<NonNullable<Row[Head]>, Rest>
        | (null extends Row[Head] ? undefined : never)
        | (undefined extends Row[Head] ? undefined : never)
    : never
  : Path extends keyof Row
    ? Row[Path]
    : never;

/** Logical alignment published on header and data cells. */
export type DataGridAlign = "center" | "end" | "start";

/** Side a column is pinned to. */
export type DataGridPinSide = "end" | "start";

/** Sort direction of one sort key. */
export type DataGridSortDirection = "ascending" | "descending";

/** Row selection policy. */
export type DataGridSelectionMode = "multiple" | "none" | "single";

/** One key of a (multi-column) sort, highest priority first. */
export interface DataGridSort {
  /** Column id this key sorts by. */
  readonly columnId: string;

  /** Direction for this key. */
  readonly direction: DataGridSortDirection;
}

/** Column ids pinned to each side, in visual order. */
export interface DataGridPinning {
  /** Columns stuck to the inline start edge. */
  readonly start: readonly string[];

  /** Columns stuck to the inline end edge. */
  readonly end: readonly string[];
}

/** Behavior shared by every column kind; `Value` is the resolved cell value. */
export interface DataGridColumnOptions<Row, Value> {
  /** Visible header label, also used for the column header's accessible name. */
  readonly header?: string;

  /** Allow sorting by this column. Display columns are never sortable. @default true */
  readonly sortable?: boolean;

  /** Allow the user to resize this column. @default true */
  readonly resizable?: boolean;

  /** Allow the user to hide this column. @default true */
  readonly hideable?: boolean;

  /** Allow the user to move this column. @default true */
  readonly reorderable?: boolean;

  /** Initial width in CSS pixels. @default 150 */
  readonly width?: number;

  /** Minimum width in CSS pixels. @default 40 */
  readonly minWidth?: number;

  /** Maximum width in CSS pixels. @default Infinity */
  readonly maxWidth?: number;

  /** Initial pin side. @default undefined */
  readonly pin?: DataGridPinSide;

  /** Logical cell alignment hook. @default "start" */
  readonly align?: DataGridAlign;

  /** Hidden until made visible. @default false */
  readonly hidden?: boolean;

  /**
   * Allow editing cells in this column; a function decides per row.
   *
   * @default false
   */
  readonly editable?: boolean | ((row: Row) => boolean);

  /** Order two values; defaults to locale-aware, `null`-last comparison. */
  compare?(left: Value, right: Value, leftRow: Row, rightRow: Row): number;

  /** Keep a row when this column's filter value is set. */
  filter?(value: Value, filterValue: unknown, row: Row): boolean;

  /** Text used for display fallbacks, the global filter, and the editor draft. */
  format?(value: Value, row: Row): string;

  /** Convert editor text back into a value. @default identity for string values */
  parse?(input: string, row: Row): Value;

  /** Return an error message to reject an edited value. */
  validate?(value: Value, row: Row): string | null;
}

/** Column reading `Row` through a typed dot path. */
export interface DataGridAccessorColumn<
  Row,
  Key extends DataGridPath<Row> = DataGridPath<Row>,
  Id extends string = string,
> extends DataGridColumnOptions<Row, DataGridPathValue<Row, Key>> {
  /** Column kind discriminant. */
  readonly kind: "accessor";

  /** Stable column id (defaults to the accessor key). */
  readonly id: Id;

  /** Dot path into the row. */
  readonly accessorKey: Key;
}

/** Column whose value is derived from the row by a function. */
export interface DataGridComputedColumn<
  Row,
  Value = unknown,
  Id extends string = string,
> extends DataGridColumnOptions<Row, Value> {
  /** Column kind discriminant. */
  readonly kind: "computed";

  /** Stable column id. */
  readonly id: Id;

  /** Derive the cell value. */
  accessor(row: Row): Value;
}

/** Column without a value (actions, selection checkboxes, drag handles). */
export interface DataGridDisplayColumn<Row, Id extends string = string> extends Omit<
  DataGridColumnOptions<Row, undefined>,
  "compare" | "editable" | "filter" | "format" | "parse" | "sortable" | "validate"
> {
  /** Column kind discriminant. */
  readonly kind: "display";

  /** Stable column id. */
  readonly id: Id;
}

/** Any column over `Row`. */
export type DataGridColumn<Row> =
  | DataGridAccessorColumn<Row>
  | DataGridComputedColumn<Row>
  | DataGridDisplayColumn<Row>;

/** Resolved value type of one column definition. */
export type DataGridColumnValue<Row, Column> =
  Column extends DataGridAccessorColumn<Row, infer Key>
    ? DataGridPathValue<Row, Key>
    : Column extends DataGridComputedColumn<Row, infer Value>
      ? Value
      : undefined;

/** Id literal of one column definition. */
export type DataGridColumnId<Column> = Column extends { readonly id: infer Id } ? Id : never;

/** One visible row after filtering, sorting, and tree flattening. */
export interface DataGridRowModel<Row> {
  /** Stable row id from `getRowId`. */
  readonly id: string;

  /** Consumer row object. */
  readonly row: Row;

  /** Zero-based index among visible rows. */
  readonly index: number;

  /** Zero-based tree depth. */
  readonly depth: number;

  /** Parent row id for nested rows. */
  readonly parentId: string | null;

  /** Whether the row has sub rows. */
  readonly expandable: boolean;

  /** Whether the row's sub rows are shown. */
  readonly expanded: boolean;

  /** Whether the row is selected. */
  readonly selected: boolean;
}

/** One visible column with resolved layout. */
export interface DataGridColumnModel<
  Row,
  Column extends DataGridColumn<Row> = DataGridColumn<Row>,
> {
  /** Column definition. */
  readonly column: Column;

  /** Column id. */
  readonly id: string;

  /** Zero-based index among visible columns. */
  readonly index: number;

  /** Current width in CSS pixels. */
  readonly width: number;

  /** Pin side, if pinned. */
  readonly pin: DataGridPinSide | null;

  /** Sticky inset from the pinned edge in CSS pixels (`0` when unpinned). */
  readonly pinOffset: number;

  /** Current sort direction, or `null` when unsorted. */
  readonly sortDirection: DataGridSortDirection | null;

  /** One-based priority among sort keys, or `null`. */
  readonly sortIndex: number | null;

  /** Whether the column can be sorted. */
  readonly sortable: boolean;
}

/** Cell coordinate; `rowId === null` addresses the header row. */
export interface DataGridCellPosition {
  /** Row id, or `null` for the header row. */
  readonly rowId: string | null;

  /** Column id. */
  readonly columnId: string;
}

/** In-progress cell edit. */
export interface DataGridEditState extends DataGridCellPosition {
  /** Row id being edited. */
  readonly rowId: string;

  /** Current editor text. */
  readonly draft: string;

  /** Validation or parse error from the last commit attempt. */
  readonly error: string | null;
}

/** Payload emitted when an edit commits. The grid never mutates rows itself. */
export interface DataGridCellEditEvent<Row> {
  /** Edited row object. */
  readonly row: Row;

  /** Edited row id. */
  readonly rowId: string;

  /** Edited column id. */
  readonly columnId: string;

  /** Parsed, validated value. */
  readonly value: unknown;

  /** Value before editing. */
  readonly previous: unknown;
}

/** Slot props for one data cell, discriminated by `columnId`. */
export type DataGridCellSlotProps<Row, Column extends DataGridColumn<Row>> = Column extends unknown
  ? {
      /** Column id discriminant: `if (props.columnId === "age") props.value` is typed. */
      readonly columnId: DataGridColumnId<Column>;
      /** Column definition. */
      readonly column: Column;
      /** Row object. */
      readonly row: Row;
      /** Row model. */
      readonly rowModel: DataGridRowModel<Row>;
      /** Resolved cell value. */
      readonly value: DataGridColumnValue<Row, Column>;
      /** Formatted text of the value. */
      readonly text: string;
      /** Whether this cell owns the roving focus. */
      readonly active: boolean;
      /** Toggle this row's selection. */
      readonly toggleSelected: () => void;
      /** Toggle this row's expansion. */
      readonly toggleExpanded: () => void;
    }
  : never;

/** Slot props for a column header. */
export interface DataGridHeaderSlotProps<Row, Column extends DataGridColumn<Row>> {
  /** Column definition. */
  readonly column: Column;

  /** Resolved column layout and sort state. */
  readonly columnModel: DataGridColumnModel<Row, Column>;

  /** Whether every visible row is selected (for select-all checkboxes). */
  readonly allSelected: boolean;

  /** Whether some but not all visible rows are selected. */
  readonly someSelected: boolean;

  /** Select or clear every visible row. */
  readonly toggleAllSelected: () => void;
}

/** Slot props for an active cell editor. */
export interface DataGridEditorSlotProps<Row> {
  /** Row object. */
  readonly row: Row;

  /** Column id. */
  readonly columnId: string;

  /** Current draft text. */
  readonly draft: string;

  /** Last validation error. */
  readonly error: string | null;

  /** Replace the draft text. */
  readonly setDraft: (value: string) => void;

  /** Parse, validate, and emit the edit. Returns whether it committed. */
  readonly commit: () => boolean;

  /** Discard the edit. */
  readonly cancel: () => void;
}
