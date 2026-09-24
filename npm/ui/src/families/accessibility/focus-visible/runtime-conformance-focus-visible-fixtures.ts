import assert from "node:assert/strict";

import { h } from "vue";

import { FocusVisibleProvider } from "./focus-visible.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const focusVisibleRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "focus-visible-provider",
    sourceFile: "families/accessibility/focus-visible/focus-visible-provider.vue",
    render: () => h(FocusVisibleProvider, null, () => h("button", { type: "button" }, "Save")),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="focus-visible-provider"/);
      assert.doesNotMatch(html, /data-vize-modality/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="focus-visible-provider"]');
      assert.ok(root instanceof HTMLElement);
      assert.ok(root.querySelector("button") instanceof HTMLButtonElement);
    },
  },
];
