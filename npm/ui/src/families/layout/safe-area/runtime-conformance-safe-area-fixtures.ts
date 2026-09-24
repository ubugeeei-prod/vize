import assert from "node:assert/strict";

import { h } from "vue";

import SafeArea from "./safe-area.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const safeAreaRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "safe-area",
    sourceFile: "families/layout/safe-area/safe-area.vue",
    render: () => h(SafeArea, { edges: ["bottom"], apply: "padding" }, { default: () => "Footer" }),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="safe-area"/);
      assert.match(html, /padding-bottom:var\(--vize-safe-area-inset-bottom\)/);
    },
    assertHydratedDom(host) {
      assert.equal(host.querySelector('[data-vize-ui="safe-area"]')?.textContent, "Footer");
    },
  },
];
