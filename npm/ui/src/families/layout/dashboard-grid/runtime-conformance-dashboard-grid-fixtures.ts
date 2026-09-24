import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import DashboardGridItem from "./dashboard-grid-item.vue";
import type { DashboardGridItemSlotState } from "./dashboard-grid-types.ts";
import DashboardGrid from "./dashboard-grid.vue";

const render = () =>
  h(
    DashboardGrid,
    { defaultLayout: [{ id: "sales", x: 0, y: 0, w: 2, h: 1 }], columns: 4, label: "Metrics" },
    () => [
      h(
        DashboardGridItem,
        { id: "sales", label: "Sales" },
        {
          default: ({ handleProps }: DashboardGridItemSlotState) =>
            h("h3", { ...handleProps }, "Sales"),
        },
      ),
    ],
  );

function assertServer(html: string): void {
  assert.match(html, /role="region" aria-label="Metrics" data-vize-ui="dashboard-grid"/);
  assert.match(html, /role="group" aria-roledescription="dashboard widget" aria-label="Sales"/);
  assert.match(html, /grid-column:1 \/ span 2;grid-row:1 \/ span 1;/);
}

function assertHydrated(host: HTMLElement): void {
  assert.equal(host.querySelector('[role="region"]')?.getAttribute("aria-label"), "Metrics");
  const widget = host.querySelector('[role="group"]');
  assert.ok(widget instanceof HTMLElement);
  assert.equal(widget.getAttribute("aria-roledescription"), "dashboard widget");
  assert.equal(host.querySelector('[data-part="drag-handle"]')?.getAttribute("tabindex"), "0");
}

export const dashboardGridRuntimeFixtures: readonly RuntimeFixture[] = (
  ["dashboard-grid", "dashboard-grid-item"] as const
).map((name) => ({
  name,
  sourceFile: `families/layout/dashboard-grid/${name}.vue`,
  render,
  assertServerMarkup: assertServer,
  assertHydratedDom: assertHydrated,
}));
