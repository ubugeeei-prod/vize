import type {
  GridAlign,
  GridAutoFlow,
  GridColumns,
  GridGap,
  GridJustify,
  GridResolvedColumns,
  GridResolvedGap,
  GridResolvedLayout,
} from "./grid-types.ts";

export const GRID_DEFAULT_COLUMNS = 1 satisfies GridColumns;
export const GRID_DEFAULT_GAP = 0 satisfies GridGap;
export const GRID_DEFAULT_ALIGN = "stretch" satisfies GridAlign;
export const GRID_DEFAULT_JUSTIFY = "stretch" satisfies GridJustify;
export const GRID_DEFAULT_AUTO_FLOW = "row" satisfies GridAutoFlow;

interface GridLayoutOptions {
  readonly align?: GridAlign | undefined;
  readonly autoFlow?: GridAutoFlow | undefined;
  readonly columnGap?: GridGap | undefined;
  readonly columns?: GridColumns | undefined;
  readonly gap?: GridGap | undefined;
  readonly justify?: GridJustify | undefined;
  readonly rowGap?: GridGap | undefined;
}

function normalizeGridColumns(columns: GridColumns): GridResolvedColumns {
  if (typeof columns === "string") return columns;
  if (!Number.isInteger(columns) || columns < 1) return "repeat(1, minmax(0, 1fr))";
  return `repeat(${columns}, minmax(0, 1fr))`;
}

function normalizeGridGap(gap: GridGap): GridResolvedGap {
  if (typeof gap === "string") return gap;
  if (!Number.isFinite(gap) || gap < 0) return "0";
  return gap === 0 ? "0" : `${gap}px`;
}

/** Resolve public Grid props into a native CSS grid contract. */
export function resolveGridLayout(options: GridLayoutOptions): GridResolvedLayout {
  const align = options.align ?? GRID_DEFAULT_ALIGN;
  const autoFlow = options.autoFlow ?? GRID_DEFAULT_AUTO_FLOW;
  const columns = normalizeGridColumns(options.columns ?? GRID_DEFAULT_COLUMNS);
  const gap = normalizeGridGap(options.gap ?? GRID_DEFAULT_GAP);
  const justify = options.justify ?? GRID_DEFAULT_JUSTIFY;
  const rowGap = normalizeGridGap(options.rowGap ?? gap);
  const columnGap = normalizeGridGap(options.columnGap ?? gap);
  const style = {
    "--vize-ui-grid-align": align,
    "--vize-ui-grid-auto-flow": autoFlow,
    "--vize-ui-grid-column-gap": columnGap,
    "--vize-ui-grid-columns": columns,
    "--vize-ui-grid-gap": gap,
    "--vize-ui-grid-justify": justify,
    "--vize-ui-grid-row-gap": rowGap,
    alignItems: "var(--vize-ui-grid-align)",
    columnGap: "var(--vize-ui-grid-column-gap)",
    display: "grid",
    gap: "var(--vize-ui-grid-gap)",
    gridAutoFlow: "var(--vize-ui-grid-auto-flow)",
    gridTemplateColumns: "var(--vize-ui-grid-columns)",
    justifyItems: "var(--vize-ui-grid-justify)",
    rowGap: "var(--vize-ui-grid-row-gap)",
  } satisfies GridResolvedLayout["style"];

  return {
    align,
    autoFlow,
    columnGap,
    columns,
    gap,
    justify,
    rowGap,
    style,
  };
}
