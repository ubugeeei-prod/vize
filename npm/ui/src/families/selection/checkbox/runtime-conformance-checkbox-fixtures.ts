import assert from "node:assert/strict";

import { h } from "vue";

import CheckboxControl from "./checkbox-control.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const checkboxRuntimeFixture: RuntimeFixture = {
  name: "checkbox",
  sourceFile: "families/selection/checkbox/checkbox-control.vue",
  render: () =>
    h(CheckboxControl, {
      ariaLabel: "Accept terms",
      defaultChecked: true,
    }),
  assertServerMarkup(html) {
    assert.match(html, /type="checkbox"/);
    assert.match(html, /aria-label="Accept terms"/);
    assert.match(html, /aria-checked="true"/);
    assert.match(html, /checked/);
  },
  assertHydratedDom(host) {
    const checkbox = host.querySelector('[data-vize-ui="checkbox"]');
    assert.ok(checkbox instanceof HTMLInputElement);
    assert.equal(checkbox.checked, true);
    assert.equal(checkbox.getAttribute("aria-checked"), "true");
  },
};
