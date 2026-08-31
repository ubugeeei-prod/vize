/** Two-axis grid virtualization composed from one row and one column virtualizer. */

import { getCurrentScope, onScopeDispose, shallowReadonly, shallowRef, watch } from "vue";
import type { MaybeRefOrGetter, ShallowRef } from "vue";

import { setupDiagnostic } from "./virtualizer-options.ts";
import { createVirtualizer } from "./virtualizer-runtime.ts";
import type {
  VirtualizerAlignment,
  VirtualizerController,
  VirtualizerOptions,
  VirtualizerRect,
} from "./virtualizer-types.ts";

/** Options shared by {@link createGridVirtualizer} and {@link useGridVirtualizer}. */
export interface GridVirtualizerOptions {
  /** Number of grid rows. Reactive values relayout on change. */
  readonly rowCount: MaybeRefOrGetter<number>;

  /** Number of grid columns. Reactive values relayout on change. */
  readonly columnCount: MaybeRefOrGetter<number>;

  /**
   * Exact row height: fixed pixels or a per-index resolver.
   *
   * @default undefined
   */
  readonly rowSize?: number | ((index: number) => number) | undefined;

  /**
   * Estimated row height used until a row is measured.
   * Required when `rowSize` is not provided.
   *
   * @default undefined
   */
  readonly estimateRowSize?: number | ((index: number) => number) | undefined;

  /**
   * Exact column width: fixed pixels or a per-index resolver.
   *
   * @default undefined
   */
  readonly columnSize?: number | ((index: number) => number) | undefined;

  /**
   * Estimated column width used until a column is measured.
   * Required when `columnSize` is not provided.
   *
   * @default undefined
   */
  readonly estimateColumnSize?: number | ((index: number) => number) | undefined;

  /**
   * Gap between adjacent rows in CSS pixels.
   *
   * @default 0
   */
  readonly rowGap?: MaybeRefOrGetter<number | undefined>;

  /**
   * Gap between adjacent columns in CSS pixels.
   *
   * @default 0
   */
  readonly columnGap?: MaybeRefOrGetter<number | undefined>;

  /**
   * Extra rows and columns rendered around the visible window.
   *
   * @default 2
   */
  readonly overscan?: MaybeRefOrGetter<number | undefined>;

  /**
   * Viewport rect assumed until a real viewport is attached and measured.
   *
   * @default { width: 0, height: 0 }
   */
  readonly initialRect?: VirtualizerRect;

  /**
   * Stable key resolver for rendered rows.
   *
   * @default the row index
   */
  readonly getRowKey?: (index: number) => string | number;

  /**
   * Stable key resolver for rendered columns.
   *
   * @default the column index
   */
  readonly getColumnKey?: (index: number) => string | number;
}

/** One rendered grid cell in viewport coordinates. */
export interface VirtualGridCell {
  /** Zero-based row index. */
  readonly rowIndex: number;

  /** Zero-based column index. */
  readonly columnIndex: number;

  /** Stable render key combining the row and column keys. */
  readonly key: string;

  /** Block-axis start offset in CSS pixels. */
  readonly top: number;

  /** Inline-axis start offset in CSS pixels. */
  readonly left: number;

  /** Cell width in CSS pixels. */
  readonly width: number;

  /** Cell height in CSS pixels. */
  readonly height: number;
}

/** Grid windowing controller over one shared scrollable viewport. */
export interface GridVirtualizerController {
  /** Row-axis virtualizer, including measurement and scrolling APIs. */
  readonly rows: VirtualizerController;

  /** Column-axis virtualizer, including measurement and scrolling APIs. */
  readonly columns: VirtualizerController;

  /** Rendered cells for the current window, in row-major order. */
  readonly virtualCells: Readonly<ShallowRef<readonly VirtualGridCell[]>>;

  /** Attach the scrollable viewport element to both axes, or detach with `null`. */
  readonly setViewport: (element: Element | null) => void;

  /** Scroll until the addressed cell satisfies the alignment on both axes. */
  readonly scrollToCell: (
    rowIndex: number,
    columnIndex: number,
    alignment?: VirtualizerAlignment,
  ) => void;

  /** Release both axes and freeze the controller. */
  readonly dispose: () => void;
}

interface AxisInputs {
  readonly count: MaybeRefOrGetter<number>;
  readonly orientation: "horizontal" | "vertical";
  readonly size?: number | ((index: number) => number) | undefined;
  readonly estimate?: number | ((index: number) => number) | undefined;
  readonly gap?: MaybeRefOrGetter<number | undefined> | undefined;
  readonly getKey?: ((index: number) => string | number) | undefined;
}

function axisOptions(options: GridVirtualizerOptions, axis: AxisInputs): VirtualizerOptions {
  return {
    count: axis.count,
    orientation: axis.orientation,
    ...(axis.size === undefined ? {} : { itemSize: axis.size }),
    ...(axis.estimate === undefined ? {} : { estimateItemSize: axis.estimate }),
    ...(axis.gap === undefined ? {} : { gap: axis.gap }),
    ...(options.overscan === undefined ? {} : { overscan: options.overscan }),
    ...(options.initialRect === undefined ? {} : { initialRect: options.initialRect }),
    ...(axis.getKey === undefined ? {} : { getItemKey: axis.getKey }),
  };
}

/** Create an SSR-safe two-axis grid virtualizer over one viewport. */
export function createGridVirtualizer(options: GridVirtualizerOptions): GridVirtualizerController {
  const rows = createVirtualizer(
    axisOptions(options, {
      count: options.rowCount,
      orientation: "vertical",
      size: options.rowSize,
      estimate: options.estimateRowSize,
      gap: options.rowGap,
      getKey: options.getRowKey,
    }),
  );
  const columns = createVirtualizer(
    axisOptions(options, {
      count: options.columnCount,
      orientation: "horizontal",
      size: options.columnSize,
      estimate: options.estimateColumnSize,
      gap: options.columnGap,
      getKey: options.getColumnKey,
    }),
  );

  const virtualCells = shallowRef<readonly VirtualGridCell[]>(Object.freeze([]));
  const stopCellWatch = watch(
    () => [rows.virtualItems.value, columns.virtualItems.value] as const,
    ([rowItems, columnItems]) => {
      const cells: VirtualGridCell[] = [];
      for (const row of rowItems) {
        for (const column of columnItems) {
          cells.push(
            Object.freeze({
              rowIndex: row.index,
              columnIndex: column.index,
              key: `${String(row.key)}:${String(column.key)}`,
              top: row.start,
              left: column.start,
              width: column.size,
              height: row.size,
            }),
          );
        }
      }
      virtualCells.value = Object.freeze(cells);
    },
    { flush: "sync", immediate: true },
  );

  return Object.freeze<GridVirtualizerController>({
    rows,
    columns,
    virtualCells: shallowReadonly(virtualCells),
    setViewport(element) {
      rows.setViewport(element);
      columns.setViewport(element);
    },
    scrollToCell(rowIndex, columnIndex, alignment = "auto") {
      rows.scrollToIndex(rowIndex, alignment);
      columns.scrollToIndex(columnIndex, alignment);
    },
    dispose() {
      stopCellWatch();
      rows.dispose();
      columns.dispose();
    },
  });
}

/** Create a grid virtualizer disposed with the current Vue effect scope. */
export function useGridVirtualizer(options: GridVirtualizerOptions): GridVirtualizerController {
  if (!getCurrentScope()) {
    throw new Error(`${setupDiagnostic}: use inside component setup or an active effect scope`);
  }
  const controller = createGridVirtualizer(options);
  onScopeDispose(controller.dispose);
  return controller;
}
