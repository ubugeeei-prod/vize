import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { resolveGridLayout } from "./grid-runtime.ts";
import type { GridExpose, GridSlotState } from "./grid.ts";
import Grid from "./grid.vue";
import { mountInteraction } from "./testing/mount.ts";

test("resolves a one-column grid with no authored CSS classes", () => {
  assert.deepEqual(resolveGridLayout({}), {
    align: "stretch",
    autoFlow: "row",
    columnGap: "0",
    columns: "repeat(1, minmax(0, 1fr))",
    gap: "0",
    justify: "stretch",
    rowGap: "0",
    style: {
      "--vize-ui-grid-align": "stretch",
      "--vize-ui-grid-auto-flow": "row",
      "--vize-ui-grid-column-gap": "0",
      "--vize-ui-grid-columns": "repeat(1, minmax(0, 1fr))",
      "--vize-ui-grid-gap": "0",
      "--vize-ui-grid-justify": "stretch",
      "--vize-ui-grid-row-gap": "0",
      alignItems: "var(--vize-ui-grid-align)",
      columnGap: "var(--vize-ui-grid-column-gap)",
      display: "grid",
      gap: "var(--vize-ui-grid-gap)",
      gridAutoFlow: "var(--vize-ui-grid-auto-flow)",
      gridTemplateColumns: "var(--vize-ui-grid-columns)",
      justifyItems: "var(--vize-ui-grid-justify)",
      rowGap: "var(--vize-ui-grid-row-gap)",
    },
  });
});

test("resolves numeric columns and gap overrides into native CSS grid values", () => {
  assert.deepEqual(
    resolveGridLayout({
      align: "baseline",
      autoFlow: "row dense",
      columnGap: "2rem",
      columns: 4,
      gap: 12,
      justify: "center",
      rowGap: 8,
    }),
    {
      align: "baseline",
      autoFlow: "row dense",
      columnGap: "2rem",
      columns: "repeat(4, minmax(0, 1fr))",
      gap: "12px",
      justify: "center",
      rowGap: "8px",
      style: {
        "--vize-ui-grid-align": "baseline",
        "--vize-ui-grid-auto-flow": "row dense",
        "--vize-ui-grid-column-gap": "2rem",
        "--vize-ui-grid-columns": "repeat(4, minmax(0, 1fr))",
        "--vize-ui-grid-gap": "12px",
        "--vize-ui-grid-justify": "center",
        "--vize-ui-grid-row-gap": "8px",
        alignItems: "var(--vize-ui-grid-align)",
        columnGap: "var(--vize-ui-grid-column-gap)",
        display: "grid",
        gap: "var(--vize-ui-grid-gap)",
        gridAutoFlow: "var(--vize-ui-grid-auto-flow)",
        gridTemplateColumns: "var(--vize-ui-grid-columns)",
        justifyItems: "var(--vize-ui-grid-justify)",
        rowGap: "var(--vize-ui-grid-row-gap)",
      },
    },
  );
});

test("falls back deliberately for invalid numeric columns and gaps", () => {
  assert.equal(resolveGridLayout({ columns: 0 }).columns, "repeat(1, minmax(0, 1fr))");
  assert.equal(resolveGridLayout({ columns: 2.5 }).columns, "repeat(1, minmax(0, 1fr))");
  assert.equal(resolveGridLayout({ gap: Number.NaN }).gap, "0");
  assert.equal(resolveGridLayout({ gap: -1 }).gap, "0");
  assert.equal(resolveGridLayout({ rowGap: Number.POSITIVE_INFINITY }).rowGap, "0");
});

test("renders a non-focusable grid by default while preserving child semantics", async () => {
  const handle = mountInteraction(Grid, {
    slots: {
      default: '<button type="button">Filter</button><a href="/docs">Docs</a>',
    },
  });
  const root = handle.root();

  assert.equal(root.tagName, "DIV");
  assert.equal(root.getAttribute("class"), null);
  assert.equal(root.getAttribute("role"), null);
  assert.equal(root.getAttribute("aria-hidden"), null);
  assert.equal(root.getAttribute("tabindex"), null);
  assert.equal(root.getAttribute("part"), "root");
  assert.equal(root.getAttribute("data-vize-ui"), "grid");
  assert.equal(root.getAttribute("data-columns"), "repeat(1, minmax(0, 1fr))");
  assert.equal(root.getAttribute("data-auto-flow"), "row");
  assert.equal(root.getAttribute("data-align"), "stretch");
  assert.equal(root.getAttribute("data-justify"), "stretch");
  assert.equal(root.style.getPropertyValue("--vize-ui-grid-columns"), "repeat(1, minmax(0, 1fr))");
  assert.equal(root.style.getPropertyValue("--vize-ui-grid-gap"), "0");
  assert.equal(root.style.getPropertyValue("--vize-ui-grid-row-gap"), "0");
  assert.equal(root.style.getPropertyValue("--vize-ui-grid-column-gap"), "0");
  assert.equal(root.style.getPropertyValue("--vize-ui-grid-align"), "stretch");
  assert.equal(root.style.getPropertyValue("--vize-ui-grid-justify"), "stretch");
  assert.equal(root.style.getPropertyValue("--vize-ui-grid-auto-flow"), "row");
  assert.equal(root.style.display, "grid");
  assert.equal(root.style.gridTemplateColumns, "var(--vize-ui-grid-columns)");
  assert.equal(root.style.gridAutoFlow, "var(--vize-ui-grid-auto-flow)");
  assert.equal(root.style.gap, "var(--vize-ui-grid-gap)");
  assert.equal(root.style.rowGap, "var(--vize-ui-grid-row-gap)");
  assert.equal(root.style.columnGap, "var(--vize-ui-grid-column-gap)");
  assert.equal(root.style.alignItems, "var(--vize-ui-grid-align)");
  assert.equal(root.style.justifyItems, "var(--vize-ui-grid-justify)");
  assert.equal(await handle.tab(), handle.getByRole("button", { name: "Filter" }));
  handle.unmount();
});

test("renders custom tracks and auto flow on a semantic host", () => {
  const handle = mountInteraction(Grid, {
    props: {
      align: "center",
      as: "section",
      autoFlow: "column dense",
      columnGap: 24,
      columns: "repeat(auto-fit, minmax(12rem, 1fr))",
      gap: "1rem",
      justify: "end",
      rowGap: "0.5rem",
    },
    attrs: {
      "aria-label": "Metric cards",
    },
    slots: {
      default: "<article>One</article><article>Two</article>",
    },
  });
  const root = handle.root();

  assert.equal(root.tagName, "SECTION");
  assert.equal(root.getAttribute("aria-label"), "Metric cards");
  assert.equal(root.getAttribute("data-auto-flow"), "column dense");
  assert.equal(root.getAttribute("data-align"), "center");
  assert.equal(root.getAttribute("data-justify"), "end");
  assert.equal(root.getAttribute("data-columns"), "repeat(auto-fit, minmax(12rem, 1fr))");
  assert.equal(
    root.style.getPropertyValue("--vize-ui-grid-columns"),
    "repeat(auto-fit, minmax(12rem, 1fr))",
  );
  assert.equal(root.style.getPropertyValue("--vize-ui-grid-gap"), "1rem");
  assert.equal(root.style.getPropertyValue("--vize-ui-grid-row-gap"), "0.5rem");
  assert.equal(root.style.getPropertyValue("--vize-ui-grid-column-gap"), "24px");
  assert.equal(root.style.getPropertyValue("--vize-ui-grid-auto-flow"), "column dense");
  assert.equal(root.children.length, 2);
  handle.unmount();
});

test("passes slot state and exposes live resolved grid state", async () => {
  const handle = mountInteraction(Grid, {
    props: {
      columns: 2,
      gap: 4,
    },
    slots: {
      default: (state: GridSlotState) =>
        `${state.columns}:${state.gap}:${state.rowGap}:${state.columnGap}:${state.align}:${state.justify}:${state.autoFlow}:${state.style.display}`,
    },
  });
  const exposed = handle.exposes<GridExpose>();

  assert.ok(exposed.element === handle.root());
  assert.equal(exposed.columns, "repeat(2, minmax(0, 1fr))");
  assert.equal(exposed.gap, "4px");
  assert.equal(exposed.rowGap, "4px");
  assert.equal(exposed.columnGap, "4px");
  assert.equal(exposed.align, "stretch");
  assert.equal(exposed.justify, "stretch");
  assert.equal(exposed.autoFlow, "row");
  assert.equal(
    handle.root().textContent,
    "repeat(2, minmax(0, 1fr)):4px:4px:4px:stretch:stretch:row:grid",
  );

  await handle.wrapper.setProps({
    align: "end",
    autoFlow: "dense",
    columnGap: "2ch",
    columns: "subgrid",
    gap: "1lh",
    justify: "start",
    rowGap: 10,
  });
  assert.equal(exposed.columns, "subgrid");
  assert.equal(exposed.gap, "1lh");
  assert.equal(exposed.rowGap, "10px");
  assert.equal(exposed.columnGap, "2ch");
  assert.equal(exposed.align, "end");
  assert.equal(exposed.justify, "start");
  assert.equal(exposed.autoFlow, "dense");
  assert.equal(handle.root().textContent, "subgrid:1lh:10px:2ch:end:start:dense:grid");
  handle.unmount();
});
