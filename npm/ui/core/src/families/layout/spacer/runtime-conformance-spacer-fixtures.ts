import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";
import Spacer from "./spacer.vue";

export const spacerRuntimeFixture: RuntimeFixture = {
  name: "spacer",
  sourceFile: "families/layout/spacer/spacer.vue",
  render: () => h(Spacer, { as: "div", blockSize: "2rem", inlineSize: "100%" }),
  assertServerMarkup(html) {
    assert.match(html, /^<div/);
    assert.match(html, /aria-hidden="true"/);
    assert.match(html, /part="root"/);
    assert.match(html, /data-vize-ui="spacer"/);
    assert.match(html, /data-state="sized"/);
    assert.match(html, /data-axis="block"/);
    assert.match(html, /data-vize-spacer-inline-size="100%"/);
    assert.match(html, /data-vize-spacer-block-size="2rem"/);
    assert.match(html, /--vize-ui-spacer-inline-size:100%/);
    assert.match(html, /--vize-ui-spacer-block-size:2rem/);
  },
  assertHydratedDom(host) {
    const spacer = host.querySelector('[data-vize-ui="spacer"]');
    assert.ok(spacer instanceof HTMLElement);
    assert.equal(spacer.getAttribute("aria-hidden"), "true");
    assert.equal(spacer.getAttribute("role"), null);
    assert.equal(spacer.getAttribute("part"), "root");
    assert.equal(spacer.getAttribute("data-axis"), "block");
    assert.equal(spacer.style.getPropertyValue("--vize-ui-spacer-inline-size"), "100%");
    assert.equal(spacer.style.getPropertyValue("--vize-ui-spacer-block-size"), "2rem");
    assert.equal(spacer.textContent, "");
  },
};
