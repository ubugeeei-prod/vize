import assert from "node:assert/strict";

import { h } from "vue";

import { ConfirmProvider } from "./confirm.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const confirmRuntimeFixture: RuntimeFixture = {
  name: "confirm-provider",
  sourceFile: "families/overlays/confirm/confirm-provider.vue",
  render: () =>
    h(ConfirmProvider, { id: "runtime-confirm" }, () => h("p", null, "Application content")),
  assertServerMarkup(html) {
    assert.match(html, /data-vize-ui="confirm-provider"/);
    assert.match(html, /data-state="idle"/);
    assert.match(html, /id="runtime-confirm"/);
    assert.doesNotMatch(html, /role="alertdialog"/);
  },
  assertHydratedDom(host) {
    const provider = host.querySelector('[data-vize-ui="confirm-provider"]');
    assert.ok(provider instanceof HTMLElement);
    assert.equal(provider.getAttribute("data-state"), "idle");
    assert.match(provider.textContent ?? "", /Application content/);
  },
};
