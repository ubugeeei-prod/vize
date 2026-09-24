import { h } from "vue";
import type { VNode } from "vue";

import { scaleBand, scaleLinear, scaleTime } from "../chart-scale/chart-scale.ts";
import ChartArea from "./chart-area.vue";
import ChartAxis from "./chart-axis.vue";
import ChartBars from "./chart-bars.vue";
import ChartCrosshair from "./chart-crosshair.vue";
import ChartDataTable from "./chart-data-table.vue";
import ChartGrid from "./chart-grid.vue";
import ChartLegend from "./chart-legend.vue";
import ChartLine from "./chart-line.vue";
import ChartPie from "./chart-pie.vue";
import ChartPoints from "./chart-points.vue";
import ChartRoot from "./chart-root.vue";
import ChartTooltip from "./chart-tooltip.vue";
import type { ChartTableColumn } from "./chart-types.ts";

/** One observation of the shared SSR and conformance fixture. */
export interface FixtureReading {
  readonly at: number;
  readonly label: string;
  readonly value: number;
}

export const fixtureReadings: readonly FixtureReading[] = [
  { at: Date.UTC(2026, 0, 1), label: "Jan", value: 12 },
  { at: Date.UTC(2026, 1, 1), label: "Feb", value: 18 },
  { at: Date.UTC(2026, 2, 1), label: "Mar", value: 9 },
];

// Instantiation expressions pin the generic SFCs so `h()` checks props against the rows.
const ReadingLine = ChartLine<FixtureReading>;
const ReadingArea = ChartArea<FixtureReading>;
const ReadingPoints = ChartPoints<FixtureReading>;
const ReadingBars = ChartBars<FixtureReading, string>;
const ReadingPie = ChartPie<FixtureReading>;
const ReadingTooltip = ChartTooltip<FixtureReading>;
const ReadingTable = ChartDataTable<FixtureReading>;
const TimeAxis = ChartAxis<Date>;
const ValueAxis = ChartAxis<number>;
const ValueGrid = ChartGrid<number>;

const columns: readonly ChartTableColumn<FixtureReading>[] = [
  { key: "label", header: "Month", rowHeader: true, value: (reading) => reading.label },
  { key: "value", header: "Value", value: (reading) => reading.value },
];

/**
 * Render a complete chart exercising every chart part with fixed geometry, so
 * server rendering, hydration, and conformance fixtures share one scene.
 */
export function renderFixtureChart(): VNode {
  const time = scaleTime({
    domain: [fixtureReadings[0]?.at ?? 0, fixtureReadings[2]?.at ?? 0],
    range: [0, 520],
  });
  const value = scaleLinear({ domain: [0, 20], range: [140, 0] });
  const band = scaleBand({
    domain: fixtureReadings.map((reading) => reading.label),
    range: [0, 520],
    padding: 0.2,
  });
  const x = (reading: FixtureReading) => time(reading.at);
  const y = (reading: FixtureReading) => value(reading.value);
  return h(
    ChartRoot,
    { height: 200, id: "readings", title: "Monthly readings", width: 600 },
    {
      default: () => [
        h(ValueGrid, { scale: value }),
        h(TimeAxis, { scale: time, tickCount: 3 }),
        h(ValueAxis, { orientation: "left", scale: value }),
        h(ReadingBars, {
          category: (reading) => reading.label,
          categoryScale: band,
          data: fixtureReadings,
          name: "Volume",
          value: (reading) => reading.value,
          valueScale: value,
        }),
        h(ReadingArea, { data: fixtureReadings, name: "Band", x, y }),
        h(ReadingLine, { curve: "monotoneX", data: fixtureReadings, name: "Trend", x, y }),
        h(ReadingPoints, {
          data: fixtureReadings,
          label: (reading) => `${reading.label} ${reading.value}`,
          name: "Readings",
          x,
          y,
        }),
        h(ReadingPie, {
          data: fixtureReadings,
          innerRadius: 10,
          outerRadius: 30,
          value: (reading) => reading.value,
        }),
        h(ChartCrosshair),
      ],
      overlay: () =>
        h(
          ReadingTooltip,
          { data: fixtureReadings },
          { default: ({ datum }: { readonly datum: FixtureReading }) => datum.label },
        ),
      after: () => [
        h(ChartLegend),
        h(ReadingTable, { caption: "Monthly readings", columns, data: fixtureReadings }),
      ],
    },
  );
}
