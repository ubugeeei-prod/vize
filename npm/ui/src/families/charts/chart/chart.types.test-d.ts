/** Compile-only assertions for the public chart component contract. */

import type { ComputedRef } from "vue";

import { scaleBand, scaleLinear, scaleTime } from "../chart-scale/chart-scale.ts";
import {
  Chart,
  ChartArea,
  ChartAxis,
  ChartBars,
  ChartCrosshair,
  ChartDataTable,
  ChartGrid,
  ChartLegend,
  ChartLine,
  ChartPie,
  ChartPoints,
  ChartRoot,
  ChartTooltip,
  useChartNavigation,
  type ChartActivePoint,
  type ChartActiveReason,
  type ChartAxisOrientation,
  type ChartRootExpose,
  type ChartSeriesKind,
  type ChartTableColumn,
} from "./chart.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Visit {
  readonly day: Date;
  readonly page: "home" | "docs";
  readonly count: number;
}

type LineProps = Parameters<typeof ChartLine<Visit>>[0];
type BarsProps = Parameters<typeof ChartBars<Visit, Visit["page"]>>[0];
type TooltipProps = Parameters<typeof ChartTooltip<Visit>>[0];
type TableProps = Parameters<typeof ChartDataTable<Visit>>[0];
type AxisProps = Parameters<typeof ChartAxis<Date>>[0];

type _Orientation = Expect<Equal<ChartAxisOrientation, "bottom" | "left" | "right" | "top">>;
type _Reason = Expect<Equal<ChartActiveReason, "keyboard" | "pointer" | "programmatic">>;
type _Kinds = Expect<Equal<ChartSeriesKind, "area" | "bars" | "line" | "pie" | "points">>;
type _LineAccessorGetsRow = Expect<Equal<Parameters<LineProps["x"]>[0], Visit>>;
type _BarsCategoryIsTyped = Expect<Equal<ReturnType<BarsProps["category"]>, "home" | "docs">>;
type _TooltipData = Expect<Equal<TooltipProps["data"], readonly Visit[]>>;
type _TableColumns = Expect<Equal<TableProps["columns"], readonly ChartTableColumn<Visit>[]>>;
type _ExposeActive = Expect<Equal<ChartRootExpose["active"], ChartActivePoint | null>>;
type _NavigationIndex = Expect<
  Equal<ReturnType<typeof useChartNavigation>["activeIndex"], ComputedRef<number | null>>
>;

const time = scaleTime({ domain: [new Date(0), new Date(86_400_000)] });
const count = scaleLinear({ domain: [0, 100], range: [100, 0] });
const pages = scaleBand({ domain: ["home", "docs"] as const, range: [0, 100] });

const axis: AxisProps = { scale: time, orientation: "left", tickCount: 4, label: "Day" };
const line: LineProps = {
  curve: "monotoneX",
  data: [],
  name: "Visits",
  x: (visit) => time(visit.day),
  y: (visit) => count(visit.count),
};
const bars: BarsProps = {
  category: (visit) => visit.page,
  categoryScale: pages,
  data: [],
  label: (visit) => `${visit.page}: ${visit.count}`,
  value: (visit) => visit.count,
  valueScale: count,
};
const table: TableProps = {
  caption: "Visits",
  columns: [{ key: "count", header: "Count", value: (visit) => visit.count }],
  data: [],
};

const badLine: LineProps = {
  data: [],
  // @ts-expect-error accessors receive typed rows.
  x: (visit) => visit.missing,
  y: () => 0,
};

const badBars: BarsProps = {
  // @ts-expect-error categories must match the band scale's domain type.
  category: () => "blog",
  categoryScale: pages,
  data: [],
  value: () => 0,
  valueScale: count,
};

// @ts-expect-error axis orientation is a closed union.
const badOrientation: ChartAxisOrientation = "middle";

const rootProps: InstanceType<typeof ChartRoot>["$props"] = {
  defaultHiddenSeries: ["Visits"],
  defaultWidth: 480,
  description: "Daily visits",
  height: 240,
  margin: { left: 48 },
  title: "Visits",
  "onUpdate:hiddenSeries": (names: readonly string[]) => names,
  onActiveChange: (point: ChartActivePoint | null, reason: ChartActiveReason) => [point, reason],
};

// @ts-expect-error charts require an accessible title.
const untitled: InstanceType<typeof ChartRoot>["$props"] = { height: 100 };

void Chart;
void ChartArea;
void ChartCrosshair;
void ChartGrid;
void ChartLegend;
void ChartPie;
void ChartPoints;
void axis;
void badBars;
void badLine;
void badOrientation;
void bars;
void line;
void rootProps;
void table;
void untitled;
