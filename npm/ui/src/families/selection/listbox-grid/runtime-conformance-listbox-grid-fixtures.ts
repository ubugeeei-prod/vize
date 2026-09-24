import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import ListboxGrid from "./listbox-grid.vue";
import ListboxGridItem from "./listbox-grid-item.vue";

const swatches = ["#ff0000", "#00ff00", "#0000ff"] as const;

function renderListboxGridFixture() {
  return h(
    ListboxGrid<string>,
    { ariaLabel: "Swatch", columns: 3, defaultValue: "#00ff00", id: "swatches", name: "swatch" },
    () =>
      swatches.map((swatch) =>
        h(ListboxGridItem<string>, { ariaLabel: swatch, key: swatch, value: swatch }),
      ),
  );
}

function assertServerMarkup(html: string): void {
  assert.match(html, /^<div id="swatches"/);
  assert.match(html, /role="listbox"/);
  assert.match(html, /data-vize-ui="listbox-grid"/);
  assert.match(html, /data-vize-ui="listbox-grid-item"/);
  assert.match(html, /aria-selected="true"/);
  assert.match(html, /name="swatch"/);
}

function assertHydratedDom(host: HTMLElement): void {
  const root = host.querySelector('[data-vize-ui="listbox-grid"]');
  assert.ok(root instanceof HTMLDivElement);
  assert.equal(root.getAttribute("role"), "listbox");
  assert.equal(host.querySelectorAll('[role="option"][aria-selected="true"]').length, 1);
}

export const listboxGridRuntimeFixtures: readonly RuntimeFixture[] = [
  "listbox-grid.vue",
  "listbox-grid-item.vue",
].map((file) => ({
  name: file.replace(".vue", ""),
  sourceFile: `families/selection/listbox-grid/${file}`,
  render: renderListboxGridFixture,
  assertServerMarkup,
  assertHydratedDom,
}));
