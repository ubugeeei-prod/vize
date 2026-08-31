import assert from "node:assert/strict";

import { h } from "vue";

import Separator from "./separator.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const separatorRuntimeFixture: RuntimeFixture = {
  name: "separator",
  sourceFile: "families/layout/separator/separator.vue",
  render: () =>
    h(Separator, {
      ariaLabel: "Pane boundary",
      as: "div",
      orientation: "vertical",
    }),
  assertServerMarkup(html) {
    assert.match(html, /^<div/);
    assert.match(html, /role="separator"/);
    assert.match(html, /aria-orientation="vertical"/);
    assert.match(html, /aria-label="Pane boundary"/);
    assert.match(html, /data-vize-ui="separator"/);
    assert.match(html, /data-state="semantic"/);
    assert.match(html, /data-orientation="vertical"/);
  },
  assertHydratedDom(host) {
    const separator = host.querySelector('[data-vize-ui="separator"]');
    assert.ok(separator instanceof HTMLElement);
    assert.equal(separator.getAttribute("role"), "separator");
    assert.equal(separator.getAttribute("aria-orientation"), "vertical");
    assert.equal(separator.getAttribute("aria-label"), "Pane boundary");
    assert.equal(separator.getAttribute("data-state"), "semantic");
  },
};
