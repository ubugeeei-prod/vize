import assert from "node:assert/strict";

import { h } from "vue";

import Alert from "./alert.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const alertRuntimeFixture: RuntimeFixture = {
  name: "alert",
  sourceFile: "families/feedback/alert/alert.vue",
  render: () =>
    h(
      Alert,
      {
        ariaDescribedby: "network-help",
        ariaLabel: "Network warning",
        id: "network-alert",
        variant: "warning",
      },
      {
        default: () => h("p", { id: "network-help" }, "Connection lost"),
      },
    ),
  assertServerMarkup(html) {
    assert.match(html, /^<div/);
    assert.match(html, /id="network-alert"/);
    assert.match(html, /role="alert"/);
    assert.match(html, /aria-live="assertive"/);
    assert.match(html, /aria-atomic="true"/);
    assert.match(html, /data-vize-ui="alert"/);
    assert.match(html, /data-state="open"/);
    assert.match(html, /data-variant="warning"/);
    assert.match(html, /Connection lost/);
  },
  assertHydratedDom(host) {
    const alert = host.querySelector('[data-vize-ui="alert"]');
    assert.ok(alert instanceof HTMLDivElement);
    assert.equal(alert.getAttribute("role"), "alert");
    assert.equal(alert.getAttribute("aria-live"), "assertive");
    assert.equal(alert.getAttribute("data-state"), "open");
    assert.equal(alert.getAttribute("data-variant"), "warning");
  },
};
