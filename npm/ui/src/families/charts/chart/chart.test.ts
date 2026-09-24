import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import { scaleBand, scaleLinear, scaleTime } from "../chart-scale/chart-scale.ts";
import type { ChartRootExpose } from "./chart.ts";
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
import { mountInteraction } from "../../../testing/mount.ts";

interface Sale {
  readonly month: string;
  readonly revenue: number;
  readonly cost: number;
}

const sales: readonly Sale[] = [
  { month: "Jan", revenue: 30, cost: 20 },
  { month: "Feb", revenue: 50, cost: 25 },
  { month: "Mar", revenue: 40, cost: 30 },
];

const plot = { width: 400, height: 200, margin: { top: 0, right: 0, bottom: 0, left: 0 } };
const x = scaleBand({ domain: sales.map((sale) => sale.month), range: [0, 300] });
const y = scaleLinear({ domain: [0, 50], range: [200, 0] });
const center = (sale: Sale) => x(sale.month) + x.bandwidth / 2;

function mountSalesChart(props: Record<string, unknown> = {}) {
  return mountInteraction(ChartRoot, {
    props: { ...plot, id: "sales", title: "Revenue by month", description: "Q1 revenue", ...props },
    record: ["activeChange", "update:hiddenSeries"],
    slots: {
      default: () => [
        h(ChartGrid, { scale: y, tickCount: 5 }),
        h(ChartAxis, { scale: x, orientation: "bottom" }),
        h(ChartAxis, { scale: y, orientation: "left", tickCount: 5, label: "Revenue" }),
        h(ChartArea, { data: sales, name: "Cost", x: center, y: (sale: Sale) => y(sale.cost) }),
        h(ChartLine, { data: sales, name: "Trend", x: center, y: (sale: Sale) => y(sale.revenue) }),
        h(ChartPoints, {
          data: sales,
          label: (sale: Sale) => `${sale.month}: ${sale.revenue}`,
          name: "Revenue",
          x: center,
          y: (sale: Sale) => y(sale.revenue),
        }),
        h(ChartCrosshair, { axis: "both" }),
      ],
      overlay: () =>
        h(
          ChartTooltip,
          { data: sales },
          {
            default: ({ datum }: { readonly datum: Sale }) => `${datum.month} ${datum.revenue}`,
          },
        ),
      after: () => [
        h(ChartLegend),
        h(ChartDataTable, {
          caption: "Revenue by month",
          columns: [
            { key: "month", header: "Month", rowHeader: true, value: (sale: Sale) => sale.month },
            { key: "revenue", header: "Revenue", value: (sale: Sale) => sale.revenue * 1000 },
          ],
          data: sales,
        }),
      ],
    },
  });
}

function points(root: HTMLElement): SVGGElement[] {
  return [...root.querySelectorAll<SVGGElement>('[data-vize-ui="chart-point"]')];
}

test("renders a labelled SVG chart with plot geometry, axes, grid, and series paths", () => {
  const handle = mountSalesChart();
  const root = handle.root();
  const svg = root.querySelector("svg");
  assert.equal(root.tagName, "FIGURE");
  assert.equal(svg?.getAttribute("role"), "group");
  assert.equal(svg?.getAttribute("aria-roledescription"), "chart");
  assert.equal(svg?.getAttribute("aria-labelledby"), "sales-title");
  assert.equal(svg?.getAttribute("aria-describedby"), "sales-description");
  assert.equal(svg?.getAttribute("viewBox"), "0 0 400 200");
  assert.equal(root.querySelector("title")?.textContent, "Revenue by month");
  const labels = [...root.querySelectorAll('[data-vize-ui="chart-axis-tick-label"]')].map((label) =>
    label.textContent?.trim(),
  );
  assert.deepEqual(labels.slice(0, 3), ["Jan", "Feb", "Mar"]);
  assert.deepEqual(labels.slice(3), ["0", "10", "20", "30", "40", "50"]);
  const bandTick = root.querySelector('[data-vize-ui="chart-axis-tick"]');
  assert.equal(bandTick?.getAttribute("transform"), "translate(50,0)", "band ticks are centered");
  assert.equal(root.querySelectorAll('[data-vize-ui="chart-grid-line"]').length, 6);
  assert.equal(
    root.querySelector('[data-vize-ui="chart-line"]')?.getAttribute("d"),
    "M50,80L150,0L250,40",
  );
  assert.equal(
    root.querySelector('[data-vize-ui="chart-area"]')?.getAttribute("d"),
    "M50,120L150,100L250,80L250,200L150,200L50,200Z",
  );
  assert.equal(
    root.querySelector('[data-vize-ui="chart-axis"]')?.getAttribute("aria-hidden"),
    "true",
  );
  handle.unmount();
});

test("points use roving focus, arrow keys, announcements, tooltip, and crosshair", async () => {
  const handle = mountSalesChart();
  const root = handle.root();
  const [jan, feb, mar] = points(root);
  assert.ok(jan && feb && mar);
  assert.equal(jan.getAttribute("tabindex"), "0");
  assert.equal(feb.getAttribute("tabindex"), "-1");
  assert.equal(jan.getAttribute("role"), "img");
  assert.equal(jan.getAttribute("aria-label"), "Jan: 30");
  assert.equal(
    root.querySelector('[data-vize-ui="chart-points"]')?.getAttribute("aria-label"),
    "Revenue",
  );

  jan.dispatchEvent(new FocusEvent("focus"));
  await nextTick();
  jan.dispatchEvent(
    new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true, cancelable: true }),
  );
  await nextTick();
  assert.equal(feb.getAttribute("data-active"), "true");
  assert.equal(feb.getAttribute("tabindex"), "0");
  assert.equal(jan.getAttribute("tabindex"), "-1");
  assert.ok(handle.activeElement() === feb, "arrow keys move DOM focus");
  assert.match(
    root.querySelector('[data-vize-ui="chart-announcer"]')?.textContent ?? "",
    /Feb: 50/,
  );
  const tooltip = root.querySelector<HTMLElement>('[data-vize-ui="chart-tooltip"]');
  assert.equal(tooltip?.hidden, false);
  assert.equal(tooltip?.textContent, "Feb 50");
  assert.equal(tooltip?.style.getPropertyValue("--vize-chart-tooltip-x"), "150px");
  assert.equal(root.querySelector('[data-vize-ui="chart-crosshair-x"]')?.getAttribute("x1"), "150");
  assert.equal(root.querySelector('[data-vize-ui="chart-crosshair-y"]')?.getAttribute("y1"), "0");

  feb.dispatchEvent(new KeyboardEvent("keydown", { key: "End", bubbles: true, cancelable: true }));
  await nextTick();
  assert.equal(mar.getAttribute("data-active"), "true");
  mar.dispatchEvent(
    new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true, cancelable: true }),
  );
  await nextTick();
  assert.equal(mar.getAttribute("data-active"), "true", "navigation stops at the ends");
  mar.dispatchEvent(new KeyboardEvent("keydown", { key: "Home", bubbles: true, cancelable: true }));
  await nextTick();
  assert.equal(jan.getAttribute("data-active"), "true");
  jan.dispatchEvent(
    new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true }),
  );
  await nextTick();
  assert.equal(tooltip?.hidden, true);
  assert.equal(
    root.querySelector('[data-vize-ui="chart-crosshair"]')?.getAttribute("data-state"),
    "hidden",
  );
  const reasons = handle.wrapper.emitted("activeChange")?.map((args) => args[1]);
  assert.ok(reasons?.every((reason) => reason === "keyboard"));
  handle.unmount();
});

test("pointer hover activates the nearest point without moving focus", async () => {
  const handle = mountSalesChart();
  const root = handle.root();
  const svg = root.querySelector("svg");
  assert.ok(svg);
  svg.getBoundingClientRect = () => new DOMRect(0, 0, 400, 200);
  const capture = root.querySelector('[data-vize-ui="chart-points-capture"]');
  assert.ok(capture);
  capture.dispatchEvent(
    new PointerEvent("pointermove", {
      clientX: 240,
      clientY: 50,
      pointerType: "mouse",
      bubbles: true,
    }),
  );
  await nextTick();
  assert.equal(points(root)[2]?.getAttribute("data-active"), "true");
  assert.equal(root.querySelector('[data-vize-ui="chart-announcer"]')?.textContent?.trim(), "");
  assert.equal(handle.wrapper.emitted("activeChange")?.at(-1)?.[1], "pointer");
  capture.dispatchEvent(new PointerEvent("pointerleave", { pointerType: "mouse" }));
  await nextTick();
  assert.equal(
    root.querySelector('[data-vize-ui="chart-tooltip"]')?.getAttribute("data-state"),
    "closed",
  );
  handle.unmount();
});

test("the legend toggles series visibility with pressed buttons", async () => {
  const handle = mountSalesChart();
  const root = handle.root();
  const toggle = handle.getByRole("button", { name: "Trend" });
  assert.deepEqual(
    [...root.querySelectorAll('[data-vize-ui="chart-legend-item"]')].map((item) =>
      item.textContent?.trim(),
    ),
    ["Cost", "Trend", "Revenue"],
  );
  assert.equal(toggle.getAttribute("aria-pressed"), "true");
  await handle.click(toggle);
  assert.equal(toggle.getAttribute("aria-pressed"), "false");
  const line = root.querySelector('[data-vize-ui="chart-line"]');
  assert.equal(line?.getAttribute("data-hidden"), "true");
  assert.equal(line?.getAttribute("d"), null);
  assert.deepEqual(handle.wrapper.emitted("update:hiddenSeries")?.[0]?.[0], ["Trend"]);

  await handle.click(handle.getByRole("button", { name: "Revenue" }));
  assert.equal(points(root).length, 0, "hidden point series leave the tab order");
  handle.unmount();

  const controlled = mountSalesChart({ hiddenSeries: ["Cost"] });
  assert.equal(
    controlled.root().querySelector('[data-vize-ui="chart-area"]')?.getAttribute("data-hidden"),
    "true",
  );
  await controlled.click(controlled.getByRole("button", { name: "Cost" }));
  assert.equal(
    controlled.root().querySelector('[data-vize-ui="chart-area"]')?.getAttribute("data-hidden"),
    "true",
    "controlled visibility waits for the parent",
  );
  controlled.unmount();
});

test("the data table fallback exposes every row with headers and localized cells", () => {
  const handle = mountSalesChart();
  const table = handle.root().querySelector("table");
  assert.ok(table);
  assert.equal(table.querySelector("caption")?.textContent?.trim(), "Revenue by month");
  assert.equal(table.getAttribute("data-visually-hidden"), "true");
  assert.equal(table.style.position, "absolute");
  assert.deepEqual(
    [...table.querySelectorAll("thead th")].map((cell) => [
      cell.textContent,
      cell.getAttribute("scope"),
    ]),
    [
      ["Month", "col"],
      ["Revenue", "col"],
    ],
  );
  const firstRow = table.querySelector("tbody tr");
  assert.equal(firstRow?.querySelector("th")?.getAttribute("scope"), "row");
  assert.equal(firstRow?.querySelector("th")?.textContent, "Jan");
  assert.equal(firstRow?.querySelector("td")?.textContent, "30,000");
  handle.unmount();
});

test("bars and pie slices join keyboard navigation when labelled", async () => {
  const handle = mountInteraction(ChartRoot, {
    props: { ...plot, title: "Mix" },
    slots: {
      default: () => [
        h(ChartBars, {
          category: (sale: Sale) => sale.month,
          categoryScale: scaleBand({
            domain: sales.map((sale) => sale.month),
            range: [0, 300],
            padding: 0.1,
          }),
          data: sales,
          label: (sale: Sale) => `${sale.month} revenue ${sale.revenue}`,
          name: "Bars",
          radius: 2,
          value: (sale: Sale) => sale.revenue,
          valueScale: y,
        }),
        h(ChartPie, {
          data: sales,
          innerRadius: 20,
          label: (sale: Sale) => `${sale.month} cost ${sale.cost}`,
          name: "Pie",
          value: (sale: Sale) => sale.cost,
        }),
      ],
    },
  });
  const root = handle.root();
  const bars = [...root.querySelectorAll<SVGPathElement>('[data-vize-ui="chart-bar"]')];
  assert.equal(bars.length, 3);
  assert.equal(bars[0]?.getAttribute("role"), "img");
  assert.equal(bars[0]?.getAttribute("tabindex"), "0");
  assert.match(bars[1]?.getAttribute("d") ?? "", /^M/);
  bars[0]?.dispatchEvent(
    new KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true, cancelable: true }),
  );
  await nextTick();
  assert.equal(bars[1]?.getAttribute("data-active"), "true");
  const slices = [...root.querySelectorAll<SVGPathElement>('[data-vize-ui="chart-slice"]')];
  assert.equal(slices.length, 3);
  assert.equal(
    root.querySelector('[data-vize-ui="chart-pie"]')?.getAttribute("transform"),
    "translate(200,100)",
  );
  slices[0]?.dispatchEvent(new FocusEvent("focus"));
  await nextTick();
  assert.equal(slices[0]?.getAttribute("data-active"), "true");
  assert.equal(bars[1]?.getAttribute("data-active"), null, "one point is active per chart");
  handle.unmount();

  const decorative = mountInteraction(ChartRoot, {
    props: { ...plot, title: "Decorative" },
    slots: {
      default: () =>
        h(ChartBars, {
          category: (sale: Sale) => sale.month,
          categoryScale: x,
          data: sales,
          value: (sale: Sale) => sale.revenue,
          valueScale: y,
        }),
    },
  });
  const group = decorative.root().querySelector('[data-vize-ui="chart-bars"]');
  assert.equal(group?.getAttribute("aria-hidden"), "true");
  assert.equal(
    decorative.root().querySelector('[data-vize-ui="chart-bar"]')?.getAttribute("tabindex"),
    null,
  );
  decorative.unmount();
});

test("responsive charts render the default width and follow ResizeObserver", async () => {
  const observers: ResizeObserverCallback[] = [];
  const original = globalThis.ResizeObserver;
  globalThis.ResizeObserver = class {
    constructor(callback: ResizeObserverCallback) {
      observers.push(callback);
    }
    observe(): void {}
    unobserve(): void {}
    disconnect(): void {}
  };
  try {
    const handle = mountInteraction(ChartRoot, {
      props: { title: "Responsive", defaultWidth: 500, height: 100 },
      slots: { default: () => null },
    });
    assert.equal(handle.root().querySelector("svg")?.getAttribute("width"), "500");
    const entry = { contentRect: { width: 812.4 } } as unknown as ResizeObserverEntry;
    observers[0]?.([entry], {} as ResizeObserver);
    await nextTick();
    assert.equal(handle.root().querySelector("svg")?.getAttribute("width"), "812");
    const expose = handle.exposes<ChartRootExpose>();
    assert.equal(expose.innerWidth, 812 - 16 - 40);
    assert.equal(expose.innerHeight, 100 - 16 - 32);
    handle.unmount();
  } finally {
    globalThis.ResizeObserver = original;
  }
});

test("time axes format ticks in the scale's zone and support custom formats and tick slots", () => {
  const time = scaleTime({
    domain: [Date.UTC(2024, 0, 1), Date.UTC(2024, 0, 3)],
    range: [0, 300],
    timeZone: "UTC",
  });
  const handle = mountInteraction(ChartRoot, {
    props: { ...plot, title: "Time" },
    slots: {
      default: () => [
        h(ChartAxis, { scale: time, tickCount: 2, orientation: "top" }),
        h(ChartAxis, {
          format: (value: number) => `${value}%`,
          orientation: "right",
          scale: y,
          tickCount: 2,
        }),
        h(
          ChartAxis,
          { scale: x, orientation: "bottom" },
          { tick: ({ label }: { readonly label: string }) => h("tspan", `[${label}]`) },
        ),
      ],
    },
  });
  const axes = [...handle.root().querySelectorAll('[data-vize-ui="chart-axis"]')];
  const text = (axis: Element | undefined) =>
    [...(axis?.querySelectorAll('[data-vize-ui="chart-axis-tick-label"]') ?? [])].map((label) =>
      label.textContent?.trim(),
    );
  assert.deepEqual(text(axes[0]), ["2024", "2 Tue", "3 Wed"]);
  assert.deepEqual(text(axes[1]), ["0%", "20%", "40%"]);
  assert.deepEqual(text(axes[2]), ["[Jan]", "[Feb]", "[Mar]"]);
  assert.equal(axes[1]?.getAttribute("transform"), "translate(400,0)");
  handle.unmount();
});

test("chart parts require a ChartRoot provider", () => {
  assert.throws(() => mountInteraction(ChartLegend), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(ChartCrosshair), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(
    () => mountInteraction(ChartTooltip, { props: { data: [] } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
});
