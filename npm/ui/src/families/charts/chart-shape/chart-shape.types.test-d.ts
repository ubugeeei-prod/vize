/** Compile-only assertions for the public chart shape contract. */

import {
  areaPath,
  barRects,
  linePath,
  pieLayout,
  stackLayout,
  type BarRect,
  type CurveName,
  type PieSlice,
  type StackOffset,
  type StackOrder,
  type StackSeries,
} from "./chart-shape.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Row {
  readonly month: string;
  readonly revenue: number;
  readonly cost: number;
}

declare const rows: readonly Row[];

const line = linePath(rows, { x: (row, index) => index, y: (row) => row.revenue, curve: "step" });
const area = areaPath(rows, { x: (_row, index) => index, y1: (row) => row.revenue, y0: 0 });
const slices = pieLayout(rows, { value: (row) => row.revenue });
const series = stackLayout(rows, { keys: ["revenue", "cost"], value: (row, key) => row[key] });
const bars = barRects(rows, {
  category: (row) => row.month,
  categoryScale: Object.assign((month: string) => month.length, { bandwidth: 10 }),
  value: (row) => row.revenue,
  valueScale: (value: number) => value,
});

type _Curves = Expect<
  Equal<CurveName, "catmullRom" | "linear" | "monotoneX" | "step" | "stepAfter" | "stepBefore">
>;
type _Orders = Expect<
  Equal<StackOrder, "appearance" | "ascending" | "descending" | "insideOut" | "none" | "reverse">
>;
type _Offsets = Expect<
  Equal<StackOffset, "diverging" | "expand" | "none" | "silhouette" | "wiggle">
>;
type _LineIsString = Expect<Equal<typeof line, string | null>>;
type _AreaIsString = Expect<Equal<typeof area, string | null>>;
type _SlicesKeepRows = Expect<Equal<typeof slices, PieSlice<Row>[]>>;
type _SeriesInferKeys = Expect<Equal<typeof series, StackSeries<Row, "revenue" | "cost">[]>>;
type _BarsKeepRows = Expect<Equal<typeof bars, BarRect<Row>[]>>;

// @ts-expect-error accessors receive the row type.
linePath(rows, { x: (row) => row.missing, y: (row) => row.cost });

// @ts-expect-error curve names are a closed union.
linePath(rows, { x: () => 0, y: () => 0, curve: "bezier" });

// @ts-expect-error stack keys must be string literals of the value accessor.
stackLayout(rows, { keys: [1], value: () => 0 });
