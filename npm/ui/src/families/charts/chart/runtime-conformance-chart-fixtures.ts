import assert from "node:assert/strict";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { renderFixtureChart } from "./chart-render-fixture.ts";

function fixture(file: string, marker: string): RuntimeFixture {
  return {
    name: marker,
    sourceFile: `families/charts/chart/${file}`,
    render: renderFixtureChart,
    assertServerMarkup(html) {
      assert.match(html, new RegExp(`data-vize-ui="${marker}"`));
      assert.match(html, /aria-roledescription="chart"/);
      assert.match(html, /aria-label="Feb 18"/);
    },
    assertHydratedDom(host) {
      assert.ok(host.querySelector(`[data-vize-ui="${marker}"]`) instanceof Element);
      const svg = host.querySelector("svg");
      assert.equal(svg?.getAttribute("aria-labelledby"), "readings-title");
      assert.equal(host.querySelectorAll('[data-vize-ui="chart-point"][tabindex="0"]').length, 1);
    },
  };
}

export const chartRuntimeFixtures: readonly RuntimeFixture[] = [
  fixture("chart-area.vue", "chart-area"),
  fixture("chart-axis.vue", "chart-axis"),
  fixture("chart-bars.vue", "chart-bars"),
  fixture("chart-crosshair.vue", "chart-crosshair"),
  fixture("chart-data-table.vue", "chart-data-table"),
  fixture("chart-grid.vue", "chart-grid"),
  fixture("chart-legend.vue", "chart-legend"),
  fixture("chart-line.vue", "chart-line"),
  fixture("chart-pie.vue", "chart-pie"),
  fixture("chart-points.vue", "chart-points"),
  fixture("chart-root.vue", "chart"),
  fixture("chart-tooltip.vue", "chart-tooltip"),
];
