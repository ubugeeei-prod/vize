import type {
  DataGridAccessorColumn,
  DataGridColumn,
  DataGridColumnOptions,
  DataGridComputedColumn,
  DataGridDisplayColumn,
  DataGridPath,
  DataGridPathValue,
} from "./data-grid-types.ts";

/** Options accepted by {@link DataGridColumnHelper.accessor}. */
export type DataGridAccessorOptions<
  Row,
  Key extends DataGridPath<Row>,
  Id extends string,
> = DataGridColumnOptions<Row, DataGridPathValue<Row, Key>> & {
  /** Column id. @default the accessor key */
  readonly id?: Id;
};

/** Options accepted by {@link DataGridColumnHelper.display}. */
export type DataGridDisplayOptions<Row> = Omit<DataGridDisplayColumn<Row>, "id" | "kind">;

/** Typed column factories bound to one row type. */
export interface DataGridColumnHelper<Row> {
  /** Column reading a typed dot path; its id is the path itself. */
  accessor<const Key extends DataGridPath<Row>>(
    accessorKey: Key,
    options?: DataGridColumnOptions<Row, DataGridPathValue<Row, Key>>,
  ): DataGridAccessorColumn<Row, Key, Key>;

  /** Column reading a typed dot path under an explicit id. */
  accessor<const Key extends DataGridPath<Row>, const Id extends string>(
    accessorKey: Key,
    options: DataGridAccessorOptions<Row, Key, Id> & { readonly id: Id },
  ): DataGridAccessorColumn<Row, Key, Id>;

  /** Column derived by a function; its return type becomes the value type. */
  computed<const Id extends string, Value>(
    id: Id,
    accessor: (row: Row) => Value,
    options?: DataGridColumnOptions<Row, Value>,
  ): DataGridComputedColumn<Row, Value, Id>;

  /** Column without a value, for actions, checkboxes, or drag handles. */
  display<const Id extends string>(
    id: Id,
    options?: DataGridDisplayOptions<Row>,
  ): DataGridDisplayColumn<Row, Id>;
}

/**
 * Create column factories for `Row`.
 *
 * ```ts
 * const column = createColumnHelper<User>();
 * const columns = [
 *   column.accessor("name"),
 *   column.accessor("address.city", { header: "City" }),
 *   column.computed("age", (user) => 2026 - user.born, { compare: (a, b) => a - b }),
 * ];
 * ```
 */
export function createColumnHelper<Row>(): DataGridColumnHelper<Row> {
  function accessor<const Key extends DataGridPath<Row>>(
    accessorKey: Key,
    options?: DataGridColumnOptions<Row, DataGridPathValue<Row, Key>>,
  ): DataGridAccessorColumn<Row, Key, Key>;
  function accessor<const Key extends DataGridPath<Row>, const Id extends string>(
    accessorKey: Key,
    options: DataGridAccessorOptions<Row, Key, Id> & { readonly id: Id },
  ): DataGridAccessorColumn<Row, Key, Id>;
  function accessor<const Key extends DataGridPath<Row>>(
    accessorKey: Key,
    options: DataGridAccessorOptions<Row, Key, string> = {},
  ): DataGridAccessorColumn<Row, Key, string> {
    const { id, ...rest } = options;
    return Object.freeze({ ...rest, kind: "accessor", accessorKey, id: id ?? accessorKey });
  }
  return {
    accessor,
    computed: (id, accessor, options) =>
      Object.freeze({ ...options, kind: "computed", id, accessor }),
    display: (id, options) => Object.freeze({ ...options, kind: "display", id }),
  };
}

/** Read a dot path at runtime, short-circuiting on `null`/`undefined`. */
export function readPath(row: unknown, path: string): unknown {
  let current: unknown = row;
  for (const segment of path.split(".")) {
    if (current === null || current === undefined || typeof current !== "object") return undefined;
    current = Reflect.get(current, segment);
  }
  return current;
}

/** Resolve the cell value of `column` for `row`. */
export function cellValue<Row>(column: DataGridColumn<Row>, row: Row): unknown {
  if (column.kind === "accessor") return readPath(row, column.accessorKey);
  if (column.kind === "computed") return column.accessor(row);
  return undefined;
}

/** Stringify a value for display fallbacks and filtering. */
export function defaultFormat(value: unknown): string {
  if (value === null || value === undefined) return "";
  if (value instanceof Date) return Number.isNaN(value.getTime()) ? "" : value.toISOString();
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "bigint" || typeof value === "boolean") {
    return String(value);
  }
  return "";
}

let collator: Intl.Collator | null = null;

function sharedCollator(): Intl.Collator {
  collator ??= new Intl.Collator(undefined, { numeric: true, sensitivity: "base" });
  return collator;
}

/** Locale-aware comparison; `null`/`undefined` always sort last. */
export function defaultCompare(left: unknown, right: unknown): number {
  const leftMissing = left === null || left === undefined;
  const rightMissing = right === null || right === undefined;
  if (leftMissing || rightMissing) return leftMissing === rightMissing ? 0 : leftMissing ? 1 : -1;
  if (typeof left === "number" && typeof right === "number") return left - right;
  if (typeof left === "bigint" && typeof right === "bigint")
    return left < right ? -1 : left > right ? 1 : 0;
  if (typeof left === "boolean" && typeof right === "boolean") return Number(left) - Number(right);
  if (left instanceof Date && right instanceof Date) return left.getTime() - right.getTime();
  return sharedCollator().compare(defaultFormat(left), defaultFormat(right));
}

/**
 * Value-erased view of a column's callbacks. Column callbacks are declared as
 * methods, so every typed column is assignable here without casts; runtime
 * values always come from the same column's accessor.
 */
export function columnCallbacks<Row>(
  column: DataGridColumn<Row>,
): DataGridColumnOptions<Row, unknown> | null {
  if (column.kind === "display") return null;
  const callbacks: DataGridColumnOptions<Row, unknown> = column;
  return callbacks;
}

/** Format one cell through the column's `format`, falling back to {@link defaultFormat}. */
export function formatCell<Row>(column: DataGridColumn<Row>, row: Row): string {
  const value = cellValue(column, row);
  const callbacks = columnCallbacks(column);
  return callbacks?.format ? callbacks.format(value, row) : defaultFormat(value);
}

/** Whether `column` is sortable. */
export function isSortable<Row>(column: DataGridColumn<Row>): boolean {
  return column.kind !== "display" && column.sortable !== false;
}

/** Whether the cell of `column` in `row` can be edited. */
export function isEditable<Row>(column: DataGridColumn<Row>, row: Row): boolean {
  if (column.kind === "display") return false;
  const editable = column.editable;
  return typeof editable === "function" ? editable(row) : editable === true;
}
