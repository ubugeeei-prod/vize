import assert from "node:assert/strict";

import { h } from "vue";

import { BackToTop } from "./back-to-top.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const backToTopRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "back-to-top",
    sourceFile: "families/actions/back-to-top/back-to-top.vue",
    render: () => h(BackToTop, { threshold: 100 }),
    assertServerMarkup(html) {
      assert.match(html, /data-vize-ui="back-to-top"/);
      assert.match(html, /hidden/);
    },
    assertHydratedDom(host) {
      const button = host.querySelector('[data-vize-ui="back-to-top"]');
      assert.ok(button instanceof HTMLButtonElement);
      assert.equal(button.getAttribute("data-state"), "hidden");
    },
  },
];
