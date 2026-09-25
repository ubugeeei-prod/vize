import { computed, nextTick, shallowRef, watch } from "vue";
import type { ComputedRef, ShallowRef } from "vue";

/** One cell of a linear period grid (months or years). */
export interface PeriodGridCell {
  /** Linear unit: `year * 12 + month - 1` for months, the ISO year for years. */
  readonly unit: number;

  /** Zero-based position on the current page. */
  readonly index: number;

  /** Whether the cell is the roving focus target. */
  readonly focused: boolean;

  /** Whether the cell holds the current value. */
  readonly selected: boolean;

  /** Whether the cell contains the resolved current period. */
  readonly current: boolean;

  /** Whether the cell is outside `min`/`max` or the grid is disabled. */
  readonly disabled: boolean;

  /** Whether the unavailability predicate rejected the cell. */
  readonly unavailable: boolean;
}

/** Inputs of {@link usePeriodGrid}. */
export interface PeriodGridOptions {
  readonly root: Readonly<ShallowRef<HTMLElement | null>>;
  readonly pageSize: number;
  readonly columns: () => number;
  /** Units moved by Shift+PageUp/PageDown. */
  readonly bigStep: number;
  readonly min: () => number | null;
  readonly max: () => number | null;
  readonly isUnavailable: (unit: number) => boolean;
  readonly selected: () => number | null;
  readonly current: () => number | null;
  readonly focusedUnit: () => number | null | undefined;
  readonly disabled: () => boolean;
  readonly readOnly: () => boolean;
  readonly direction: () => "ltr" | "rtl";
  readonly onFocusChange: (unit: number) => void;
  readonly onSelect: (unit: number, event: Event) => void;
}

/** Page start (first unit) containing `unit`. */
export function periodPageStart(unit: number, pageSize: number): number {
  return Math.floor(unit / pageSize) * pageSize;
}

/** Unit offset requested by a key in a grid of `columns`, or `null` when unhandled. */
export function periodGridKeyOffset(
  key: string,
  options: {
    readonly index: number;
    readonly columns: number;
    readonly pageSize: number;
    readonly bigStep: number;
    readonly shiftKey: boolean;
    readonly rtl: boolean;
  },
): number | null {
  const column = options.index % options.columns;
  switch (key) {
    case "ArrowLeft":
      return options.rtl ? 1 : -1;
    case "ArrowRight":
      return options.rtl ? -1 : 1;
    case "ArrowUp":
      return -options.columns;
    case "ArrowDown":
      return options.columns;
    case "Home":
      return -column;
    case "End":
      return options.columns - 1 - column;
    case "PageUp":
      return -(options.shiftKey ? options.bigStep : options.pageSize);
    case "PageDown":
      return options.shiftKey ? options.bigStep : options.pageSize;
    default:
      return null;
  }
}

function clampUnit(unit: number, min: number | null, max: number | null): number {
  if (min !== null && unit < min) return min;
  if (max !== null && unit > max) return max;
  return unit;
}

/**
 * Roving-focus grid over a linear sequence of periods split into pages, shared
 * by MonthPicker (12 months per page) and YearPicker (`pageSize` years).
 */
export function usePeriodGrid(options: PeriodGridOptions) {
  const internalFocused = shallowRef<number | null>(null);
  const bounds = computed(() => {
    const min = options.min();
    const max = options.max();
    return min !== null && max !== null && min > max ? { min: max, max: min } : { min, max };
  });
  const focusedUnit = computed<number | null>(() => {
    const candidate =
      options.focusedUnit() ?? internalFocused.value ?? options.selected() ?? options.current();
    return candidate === null || candidate === undefined
      ? null
      : clampUnit(Math.trunc(candidate), bounds.value.min, bounds.value.max);
  });
  const pageStart = computed(() =>
    focusedUnit.value === null ? null : periodPageStart(focusedUnit.value, options.pageSize),
  );
  const columns = computed(() => {
    const value = Math.trunc(options.columns());
    return Number.isFinite(value) && value >= 1 && value <= options.pageSize ? value : 3;
  });

  watch(options.selected, (selected) => {
    if (selected !== null && pageStart.value !== periodPageStart(selected, options.pageSize)) {
      internalFocused.value = selected;
    }
  });

  function isOutOfRange(unit: number): boolean {
    const { min, max } = bounds.value;
    return (min !== null && unit < min) || (max !== null && unit > max);
  }

  const cells = computed<readonly PeriodGridCell[]>(() => {
    const start = pageStart.value;
    if (start === null) return [];
    const selected = options.selected();
    const current = options.current();
    return Array.from({ length: options.pageSize }, (_, index): PeriodGridCell => {
      const unit = start + index;
      const disabled = options.disabled() || isOutOfRange(unit);
      return {
        unit,
        index,
        focused: unit === focusedUnit.value,
        selected: unit === selected,
        current: unit === current,
        disabled,
        unavailable: !disabled && options.isUnavailable(unit),
      };
    });
  });
  const rows = computed<readonly (readonly PeriodGridCell[])[]>(() => {
    const result: PeriodGridCell[][] = [];
    for (const cell of cells.value) {
      const row = Math.floor(cell.index / columns.value);
      (result[row] ??= []).push(cell);
    }
    return result;
  });

  function cellElement(unit: number): HTMLElement | null {
    const element = options.root.value?.querySelector(`[data-unit="${unit}"]`);
    return element instanceof HTMLElement ? element : null;
  }

  function focus(focusOptions?: FocusOptions): boolean {
    const unit = focusedUnit.value;
    const target = unit === null ? null : cellElement(unit);
    if (!target || target.hasAttribute("disabled")) return false;
    target.focus(focusOptions);
    return true;
  }

  function setFocusedUnit(unit: number, moveFocus = false): void {
    const next = clampUnit(Math.trunc(unit), bounds.value.min, bounds.value.max);
    const previous = focusedUnit.value;
    internalFocused.value = next;
    if (previous !== next) options.onFocusChange(next);
    if (moveFocus) void nextTick(() => focus());
  }

  function canPage(step: -1 | 1): boolean {
    const start = pageStart.value;
    if (start === null || options.disabled()) return false;
    const nextStart = start + step * options.pageSize;
    const { min, max } = bounds.value;
    if (min !== null && nextStart + options.pageSize - 1 < min) return false;
    if (max !== null && nextStart > max) return false;
    return true;
  }

  function page(step: -1 | 1): boolean {
    const focused = focusedUnit.value;
    if (focused === null || !canPage(step)) return false;
    setFocusedUnit(focused + step * options.pageSize);
    return true;
  }

  function onCellKeydown(cell: PeriodGridCell, event: KeyboardEvent): void {
    if (options.disabled()) return;
    const offset = periodGridKeyOffset(event.key, {
      index: cell.index,
      columns: columns.value,
      pageSize: options.pageSize,
      bigStep: options.bigStep,
      shiftKey: event.shiftKey,
      rtl: options.direction() === "rtl",
    });
    if (offset === null) return;
    event.preventDefault();
    setFocusedUnit(cell.unit + offset, true);
  }

  function onCellClick(cell: PeriodGridCell, event: MouseEvent): void {
    if (cell.disabled) return;
    setFocusedUnit(cell.unit);
    if (cell.unavailable || options.readOnly()) return;
    options.onSelect(cell.unit, event);
  }

  return {
    focusedUnit,
    pageStart,
    cells,
    rows,
    columns,
    canPage,
    page,
    focus,
    setFocusedUnit,
    onCellKeydown,
    onCellClick,
  } satisfies Record<string, ComputedRef<unknown> | ((...args: never[]) => unknown)>;
}
