/**
 * Pure value and 2-D navigation helpers for ListboxGrid.
 *
 * Everything here is DOM-free and deterministic so SSR, hydration, and tests
 * observe the same results. The emoji picker reuses the grid navigation.
 */

/** Public model: `T | null` in single mode, `readonly T[]` when `Multiple` is `true`. */
export type ListboxGridModelValue<T, Multiple extends boolean> = Multiple extends true
  ? readonly T[]
  : T | null;

/** Property keys of object values usable as a comparison key. */
export type ListboxGridValueKey<T> = T extends object ? Extract<keyof T, string> : never;

/** Compare values by a property key or with a custom equality function. */
export type ListboxGridBy<T> = ListboxGridValueKey<T> | ((left: T, right: T) => boolean);

/** Movement requested inside a 2-D grid. */
export type GridMove =
  | "down"
  | "first"
  | "last"
  | "left"
  | "page-down"
  | "page-up"
  | "right"
  | "row-end"
  | "row-start"
  | "up";

/** Options for {@link moveInGrid}. */
export interface GridMoveOptions {
  /** Current index, or `-1` when nothing is active. */
  readonly index: number;
  /** Total number of cells. */
  readonly count: number;
  /** Cells per row (at least 1). */
  readonly columns: number;
  /** Reading direction; `rtl` mirrors left/right. */
  readonly direction?: "ltr" | "rtl";
  /** Rows traversed by PageUp/PageDown. @default 3 */
  readonly pageRows?: number;
  /** Wrap left/right across row boundaries. @default true */
  readonly wrapRows?: boolean;
  /** Whether a cell can receive the active state. @default every cell */
  readonly isEnabled?: (index: number) => boolean;
}

function readKey(value: unknown, key: string): unknown {
  return typeof value === "object" && value !== null ? Reflect.get(value, key) : value;
}

/** Resolve `by` into one equality function. */
export function createGridEquality<T>(by: ListboxGridBy<T> | undefined): (a: T, b: T) => boolean {
  if (by === undefined) return Object.is;
  if (typeof by === "function") return by;
  const key: string = by;
  return (left, right) =>
    Object.is(left, right) || Object.is(readKey(left, key), readKey(right, key));
}

function isList<T>(value: readonly T[] | T): value is readonly T[] {
  return Array.isArray(value);
}

/** Normalize a public model value into a readonly selection list. */
export function toGridSelection<T>(
  value: readonly T[] | T | null | undefined,
  multiple: boolean,
): readonly T[] {
  if (value === null || value === undefined) return Object.freeze([]);
  if (multiple && isList(value)) return Object.freeze([...value]);
  // A single-mode value is exactly one option value, even when `T` is an array.
  return Object.freeze([value as T]);
}

/** Convert a selection list to the public model for `multiple`. */
export function fromGridSelection<T, Multiple extends boolean>(
  values: readonly T[],
  multiple: boolean,
): ListboxGridModelValue<T, Multiple> {
  const model: readonly T[] | T | null = multiple
    ? Object.freeze([...values])
    : (values[0] ?? null);
  return model as ListboxGridModelValue<T, Multiple>;
}

/** Serialize a value for form submission. */
export function serializeGridValue<T>(value: T, by: ListboxGridBy<T> | undefined): string {
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean" || typeof value === "bigint") {
    return String(value);
  }
  if (typeof by === "string") return serializeGridValue(readKey(value, by), undefined);
  return JSON.stringify(value) ?? "";
}

function clampColumns(columns: number): number {
  return Number.isFinite(columns) && columns >= 1 ? Math.floor(columns) : 1;
}

function scan(
  start: number,
  step: number,
  count: number,
  isEnabled: (index: number) => boolean,
  limit: (index: number) => boolean,
): number | null {
  for (let index = start; index >= 0 && index < count && limit(index); index += step) {
    if (isEnabled(index)) return index;
  }
  return null;
}

/**
 * Compute the next active index for a 2-D movement.
 *
 * Vertical moves keep the column and skip disabled cells in that column;
 * horizontal moves skip disabled cells in reading order. Returns the current
 * index when no enabled target exists, or `null` for an empty grid.
 */
export function moveInGrid(move: GridMove, options: GridMoveOptions): number | null {
  const { count } = options;
  if (count <= 0) return null;
  const columns = clampColumns(options.columns);
  const isEnabled = options.isEnabled ?? (() => true);
  const rtl = options.direction === "rtl";
  const current = options.index;
  const any = () => true;
  const firstEnabled = () => scan(0, 1, count, isEnabled, any);
  const lastEnabled = () => scan(count - 1, -1, count, isEnabled, any);
  if (current < 0 || current >= count) {
    return move === "last" || move === "up" || move === "page-up" ? lastEnabled() : firstEnabled();
  }
  const row = Math.floor(current / columns);
  const rowStart = row * columns;
  const rowEnd = Math.min(count - 1, rowStart + columns - 1);
  const keep = (target: number | null) => target ?? current;
  const horizontal = move === "left" || move === "right";
  const forward = horizontal ? (move === "right") !== rtl : false;
  switch (move) {
    case "left":
    case "right": {
      const step = forward ? 1 : -1;
      const inRow = (index: number) => index >= rowStart && index <= rowEnd;
      return keep(
        scan(current + step, step, count, isEnabled, options.wrapRows === false ? inRow : any),
      );
    }
    case "down":
    case "up": {
      const step = move === "down" ? columns : -columns;
      return keep(scan(current + step, step, count, isEnabled, any));
    }
    case "page-down":
    case "page-up": {
      const rows = Math.max(1, Math.floor(options.pageRows ?? 3));
      const column = current % columns;
      if (move === "page-down") {
        const lastInColumn = column + Math.floor((count - 1 - column) / columns) * columns;
        const target = Math.min(current + columns * rows, lastInColumn);
        return keep(scan(target, -columns, count, isEnabled, (index) => index > current));
      }
      const target = Math.max(current - columns * rows, column);
      return keep(scan(target, columns, count, isEnabled, (index) => index < current));
    }
    case "row-start":
      return keep(scan(rowStart, 1, count, isEnabled, (index) => index <= rowEnd));
    case "row-end":
      return keep(scan(rowEnd, -1, count, isEnabled, (index) => index >= rowStart));
    case "first":
      return keep(firstEnabled());
    case "last":
      return keep(lastEnabled());
  }
}

/** Map a keyboard event to a grid movement, or `null` for other keys. */
export function gridMoveFromKey(event: KeyboardEvent): GridMove | null {
  if (event.altKey || event.metaKey || event.isComposing) return null;
  switch (event.key) {
    case "ArrowDown":
      return "down";
    case "ArrowUp":
      return "up";
    case "ArrowLeft":
      return "left";
    case "ArrowRight":
      return "right";
    case "PageDown":
      return "page-down";
    case "PageUp":
      return "page-up";
    case "Home":
      return event.ctrlKey ? "first" : "row-start";
    case "End":
      return event.ctrlKey ? "last" : "row-end";
    default:
      return null;
  }
}
