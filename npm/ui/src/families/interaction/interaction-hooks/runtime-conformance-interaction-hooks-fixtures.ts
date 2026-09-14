import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import InteractionHooksExample from "./interaction-hooks-example.vue";

export const interactionHooksRuntimeFixture: RuntimeFixture = {
  name: "interaction-hooks-example",
  sourceFile: "families/interaction/interaction-hooks/interaction-hooks-example.vue",
  render: () => h(InteractionHooksExample),
  assertServerMarkup(html) {
    assert.match(html, /^<button/);
    assert.match(html, /data-vize-ui="interaction-hooks-example"/);
    assert.match(html, /type="button"/);
    assert.match(html, /Activated 0 times/);
    assert.doesNotMatch(html, /data-focused=/);
    assert.doesNotMatch(html, /data-pressed=/);
  },
  assertHydratedDom(host) {
    const button = host.querySelector('[data-vize-ui="interaction-hooks-example"]');
    assert.ok(button instanceof HTMLButtonElement);
    assert.equal(button.type, "button");
    assert.equal(button.textContent?.trim(), "Activated 0 times");
  },
};
