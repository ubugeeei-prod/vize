import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import ToggleButton from "./toggle-button.vue";

export const toggleRuntimeFixture: RuntimeFixture = {
  name: "toggle",
  sourceFile: "families/selection/toggle/toggle-button.vue",
  render: () =>
    h(
      ToggleButton,
      { defaultPressed: true },
      {
        default: () => "Bold",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<button/);
    assert.match(html, /type="button"/);
    assert.match(html, /aria-pressed="true"/);
    assert.match(html, /data-vize-ui="toggle"/);
    assert.match(html, /data-state="pressed"/);
    assert.match(html, /Bold/);
  },
  assertHydratedDom(host) {
    const toggle = host.querySelector('[data-vize-ui="toggle"]');
    assert.ok(toggle instanceof HTMLButtonElement);
    assert.equal(toggle.type, "button");
    assert.equal(toggle.getAttribute("aria-pressed"), "true");
    assert.equal(toggle.getAttribute("data-state"), "pressed");
    assert.equal(toggle.textContent, "Bold");
  },
};
