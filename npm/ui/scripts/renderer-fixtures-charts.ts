export const chartRendererFixtures = [
  {
    filename: "ChartConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { scaleBand, scaleLinear } from "./families/charts/chart-scale/chart-scale.ts";
import {
  ChartAxis,
  ChartBars,
  ChartCrosshair,
  ChartDataTable,
  ChartGrid,
  ChartLegend,
  ChartLine,
  ChartPoints,
  ChartRoot,
  ChartTooltip,
} from "./families/charts/chart/chart.ts";

interface Sale {
  readonly month: string;
  readonly revenue: number;
}

const sales: readonly Sale[] = [
  { month: "Jan", revenue: 30 },
  { month: "Feb", revenue: 50 },
];
const x = scaleBand({ domain: ["Jan", "Feb"], range: [0, 300], padding: 0.2 });
const y = scaleLinear({ domain: [0, 60], range: [200, 0] });
const center = (sale: Sale) => x(sale.month) + x.bandwidth / 2;
const columns = [{ key: "revenue", header: "Revenue", value: (sale: Sale) => sale.revenue }];
</script>

<template>
  <ChartRoot title="Revenue" :height="240">
    <ChartGrid :scale="y" />
    <ChartAxis :scale="x" />
    <ChartAxis :scale="y" orientation="left" label="USD" />
    <ChartBars
      :data="sales"
      :category="(sale) => sale.month"
      :category-scale="x"
      :value="(sale) => sale.revenue"
      :value-scale="y"
      name="Revenue"
    />
    <ChartLine :data="sales" :x="center" :y="(sale) => y(sale.revenue)" curve="monotoneX" name="Trend" />
    <ChartPoints
      :data="sales"
      :x="center"
      :y="(sale) => y(sale.revenue)"
      :label="(sale) => sale.month + ': ' + sale.revenue"
      name="Points"
    />
    <ChartCrosshair />
    <template #overlay>
      <ChartTooltip :data="sales">
        <template #default="{ datum }">{{ datum.month }}</template>
      </ChartTooltip>
    </template>
    <template #after>
      <ChartLegend />
      <ChartDataTable :data="sales" :columns="columns" caption="Revenue" />
    </template>
  </ChartRoot>
</template>
`,
  },
] as const;
