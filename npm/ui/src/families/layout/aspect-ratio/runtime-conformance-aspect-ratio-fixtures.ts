import assert from "node:assert/strict";

import { h } from "vue";

import AspectRatio from "./aspect-ratio.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const aspectRatioRuntimeFixture: RuntimeFixture = {
  name: "aspect-ratio",
  sourceFile: "families/layout/aspect-ratio/aspect-ratio.vue",
  render: () =>
    h(
      AspectRatio,
      { as: "figure", ratio: 16 / 9 },
      {
        default: () => "Poster",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<figure/);
    assert.match(html, /data-vize-ui="aspect-ratio"/);
    assert.match(html, /data-state="valid"/);
    assert.match(html, /data-vize-aspect-ratio="1\.7777777777777777"/);
    assert.match(html, /Poster/);
  },
  assertHydratedDom(host) {
    const box = host.querySelector('[data-vize-ui="aspect-ratio"]');
    assert.ok(box instanceof HTMLElement);
    assert.equal(box.tagName, "FIGURE");
    assert.equal(box.getAttribute("data-state"), "valid");
    assert.equal(box.style.getPropertyValue("--vize-ui-aspect-ratio"), "1.7777777777777777");
    assert.equal(box.textContent, "Poster");
  },
};
