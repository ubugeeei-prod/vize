import assert from "node:assert/strict";

import { h } from "vue";

import SwitchControl from "./switch-control.vue";
import type { RuntimeFixture } from "./runtime-conformance-fixtures.ts";

export const switchRuntimeFixture: RuntimeFixture = {
  name: "switch",
  sourceFile: "switch-control.vue",
  render: () =>
    h(
      SwitchControl,
      {
        ariaLabel: "Notifications",
        defaultChecked: true,
        id: "notifications",
        name: "notifications",
        required: true,
        value: "email",
      },
      {
        default: () => "Notifications",
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<button/);
    assert.match(html, /id="notifications"/);
    assert.match(html, /role="switch"/);
    assert.match(html, /aria-checked="true"/);
    assert.match(html, /aria-required="true"/);
    assert.match(html, /data-vize-ui="switch"/);
    assert.match(html, /data-state="checked"/);
    assert.match(html, /data-checked="true"/);
    assert.match(html, /<input type="hidden" name="notifications" value="email"/);
  },
  assertHydratedDom(host) {
    const control = host.querySelector('[data-vize-ui="switch"]');
    assert.ok(control instanceof HTMLButtonElement);
    assert.equal(control.type, "button");
    assert.equal(control.getAttribute("role"), "switch");
    assert.equal(control.getAttribute("aria-checked"), "true");
    assert.equal(control.getAttribute("data-checked"), "true");
    const hidden = control.querySelector("input[type='hidden']");
    assert.ok(hidden instanceof HTMLInputElement);
    assert.equal(hidden.name, "notifications");
    assert.equal(hidden.value, "email");
  },
};
