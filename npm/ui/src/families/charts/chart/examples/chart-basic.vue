<!-- Line chart with keyboard-navigable points, axes, a tooltip, and a data-table fallback. -->
<script setup lang="ts">
import {
  Chart,
  ChartAxis,
  ChartDataTable,
  ChartGrid,
  ChartLine,
  ChartPoints,
  ChartTooltip,
} from "../chart.ts";
import type { ChartAxisScale, ChartTableColumn } from "../chart.ts";

interface Visits {
  readonly month: string;
  readonly count: number;
}

const visits: readonly Visits[] = [
  { month: "Jan", count: 1200 },
  { month: "Feb", count: 1850 },
  { month: "Mar", count: 1600 },
  { month: "Apr", count: 2400 },
  { month: "May", count: 2100 },
];

// Plot area of a 480×240 chart with the default margins: 424×192.
const months = visits.map((row) => row.month);
const monthScale: ChartAxisScale<string> = Object.assign(
  (month: string) => (months.indexOf(month) * 424) / (months.length - 1),
  { range: [0, 424], ticks: () => [...months] },
);
const countScale: ChartAxisScale<number> = Object.assign(
  (count: number) => 192 - (count / 3000) * 192,
  {
    range: [192, 0],
    ticks: () => [0, 1000, 2000, 3000],
  },
);

const x = (row: Visits) => monthScale(row.month);
const y = (row: Visits) => countScale(row.count);
const label = (row: Visits) => `${row.month}: ${row.count} visits`;
const columns: readonly ChartTableColumn<Visits>[] = [
  { key: "month", header: "Month", rowHeader: true, value: (row) => row.month },
  { key: "count", header: "Visits", value: (row) => row.count },
];
</script>

<template>
  <Chart
    title="Monthly visits"
    description="Site visits from January to May."
    :width="480"
    :height="240"
  >
    <ChartGrid :scale="countScale" />
    <ChartAxis :scale="monthScale" />
    <ChartAxis :scale="countScale" orientation="left" />
    <ChartLine :data="visits" :x :y name="Visits" />
    <ChartPoints :data="visits" :x :y :label name="Visits" />
    <template #overlay>
      <ChartTooltip v-slot="{ datum }" :data="visits">{{ label(datum) }}</ChartTooltip>
    </template>
    <template #after>
      <ChartDataTable :data="visits" :columns caption="Monthly visits" />
    </template>
  </Chart>
</template>
