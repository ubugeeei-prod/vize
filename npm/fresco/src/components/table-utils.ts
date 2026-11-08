import { graphemes } from "../utils/text.js";
import type {
  TableAlignment,
  TableColumn,
  TableRow,
  TableSortDirection,
  TableViewportRange,
} from "./table-types.js";

export interface IndexedTableRow<Row extends TableRow = TableRow> {
  readonly row: Row;
  readonly sourceIndex: number;
}

function finiteInteger(value: number, fallback: number): number {
  return Number.isFinite(value) ? Math.trunc(value) : fallback;
}

/** Resolve a controlled or declared numeric width inside the column limits. */
export function tableColumnWidth(
  column: TableColumn,
  widths: Readonly<Record<string, number>> | undefined,
): number | undefined {
  const requested = widths?.[String(column.key)] ?? column.width;
  if (typeof requested !== "number" || !Number.isFinite(requested)) return undefined;
  const minimum = Math.max(1, finiteInteger(column.minWidth ?? 1, 1));
  const maximum = Math.max(
    minimum,
    finiteInteger(column.maxWidth ?? Number.MAX_SAFE_INTEGER, minimum),
  );
  return Math.min(maximum, Math.max(minimum, Math.trunc(requested)));
}

/** Convert supported primitive values and structured values into table text. */
export function stringifyTableCell(value: unknown): string {
  if (value == null) return "";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean" || typeof value === "bigint") {
    return String(value);
  }
  try {
    return JSON.stringify(value) ?? "";
  } catch {
    return "";
  }
}

/** Truncate and align a cell without splitting a Unicode grapheme cluster. */
export function fitTableCell(
  value: string,
  width: number | undefined,
  alignment: TableAlignment = "left",
): string {
  if (width === undefined) return value;
  const size = Math.max(0, Math.trunc(width));
  const parts = graphemes(value);
  if (parts.length >= size) return parts.slice(0, size).join("");
  const padding = size - parts.length;
  if (alignment === "right") return `${" ".repeat(padding)}${value}`;
  if (alignment === "center") {
    const left = Math.floor(padding / 2);
    return `${" ".repeat(left)}${value}${" ".repeat(padding - left)}`;
  }
  return `${value}${" ".repeat(padding)}`;
}

function compareTableValues(left: unknown, right: unknown): number {
  if (Object.is(left, right)) return 0;
  if (left == null) return -1;
  if (right == null) return 1;
  if (typeof left === "number" && typeof right === "number") {
    if (Number.isNaN(left)) return Number.isNaN(right) ? 0 : -1;
    if (Number.isNaN(right)) return 1;
    return left - right;
  }
  return stringifyTableCell(left).localeCompare(stringifyTableCell(right));
}

/** Stable controlled sorting that retains each row's original source index. */
export function sortTableRows<Row extends TableRow>(
  rows: readonly Row[],
  columns: readonly TableColumn<Row>[],
  sortKey: string | undefined,
  direction: TableSortDirection,
): IndexedTableRow<Row>[] {
  const indexed = rows.map((row, sourceIndex) => ({ row, sourceIndex }));
  if (sortKey === undefined) return indexed;
  const column = columns.find(({ key }) => String(key) === sortKey);
  if (!column) return indexed;
  const factor = direction === "descending" ? -1 : 1;
  return indexed.toSorted((left, right) => {
    const compared = column.compare
      ? column.compare(left.row, right.row)
      : compareTableValues(left.row[sortKey], right.row[sortKey]);
    return compared === 0 ? left.sourceIndex - right.sourceIndex : compared * factor;
  });
}

/** Clamp visible and retained row ranges for bounded rendering. */
export function tableViewportRange(
  itemCount: number,
  offset: number,
  viewportSize: number | undefined,
  overscan: number,
): TableViewportRange {
  const count = Math.max(0, finiteInteger(itemCount, 0));
  const visibleSize =
    viewportSize === undefined
      ? count
      : Math.min(count, Math.max(0, finiteInteger(viewportSize, 0)));
  const visibleStart = Math.min(
    Math.max(0, count - visibleSize),
    Math.max(0, finiteInteger(offset, 0)),
  );
  const visibleEnd = Math.min(count, visibleStart + visibleSize);
  const retainedMargin = Math.max(0, finiteInteger(overscan, 0));
  return {
    visibleStart,
    visibleEnd,
    retainedStart: Math.max(0, visibleStart - retainedMargin),
    retainedEnd: Math.min(count, visibleEnd + retainedMargin),
  };
}
