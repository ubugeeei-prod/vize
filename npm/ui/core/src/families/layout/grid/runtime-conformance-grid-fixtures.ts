import assert from "node:assert/strict";

import { h } from "vue";

import Grid from "./grid.vue";
import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";

export const gridRuntimeFixture: RuntimeFixture = {
  name: "grid",
  sourceFile: "families/layout/grid/grid.vue",
  render: () =>
    h(
      Grid,
      {
        align: "center",
        as: "section",
        autoFlow: "row dense",
        columnGap: "1rem",
        columns: 3,
        gap: 6,
        justify: "end",
      },
      {
        default: () => [h("article", "Alpha"), h("article", "Beta")],
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<section/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="grid"/);
    assert.match(html, /data-columns="repeat\(3, minmax\(0, 1fr\)\)"/);
    assert.match(html, /data-auto-flow="row dense"/);
    assert.match(html, /data-align="center"/);
    assert.match(html, /data-justify="end"/);
    assert.match(html, /--vize-ui-grid-columns:repeat\(3, minmax\(0, 1fr\)\)/);
    assert.match(html, /--vize-ui-grid-gap:6px/);
    assert.match(html, /--vize-ui-grid-column-gap:1rem/);
    assert.match(html, /display:grid/);
    assert.match(html, /grid-template-columns:var\(--vize-ui-grid-columns\)/);
    assert.match(html, /grid-auto-flow:var\(--vize-ui-grid-auto-flow\)/);
    assert.match(html, /<article>Alpha<\/article><article>Beta<\/article>/);
  },
  assertHydratedDom(host) {
    const grid = host.querySelector('[data-vize-ui="grid"]');
    assert.ok(grid instanceof HTMLElement);
    assert.equal(grid.getAttribute("role"), null);
    assert.equal(grid.getAttribute("aria-hidden"), null);
    assert.equal(grid.getAttribute("tabindex"), null);
    assert.equal(grid.getAttribute("part"), "root");
    assert.equal(grid.getAttribute("data-columns"), "repeat(3, minmax(0, 1fr))");
    assert.equal(grid.getAttribute("data-auto-flow"), "row dense");
    assert.equal(grid.getAttribute("data-align"), "center");
    assert.equal(grid.getAttribute("data-justify"), "end");
    assert.equal(
      grid.style.getPropertyValue("--vize-ui-grid-columns"),
      "repeat(3, minmax(0, 1fr))",
    );
    assert.equal(grid.style.getPropertyValue("--vize-ui-grid-gap"), "6px");
    assert.equal(grid.style.getPropertyValue("--vize-ui-grid-column-gap"), "1rem");
    assert.equal(grid.style.display, "grid");
    assert.equal(grid.style.gridTemplateColumns, "var(--vize-ui-grid-columns)");
    assert.equal(grid.style.gridAutoFlow, "var(--vize-ui-grid-auto-flow)");
    assert.equal(grid.textContent, "AlphaBeta");
  },
};
