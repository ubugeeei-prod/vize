import assert from "node:assert/strict";

import { h } from "vue";

import { Sticky } from "./sticky.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const stickyRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "sticky",
    sourceFile: "families/layout/sticky/sticky.vue",
    render: () => h(Sticky, { as: "header", offset: 4 }, () => "Toolbar"),
    assertServerMarkup(html) {
      assert.match(html, /<header[^>]*data-vize-ui="sticky"/);
      assert.match(html, /position:sticky/);
    },
    assertHydratedDom(host) {
      const sticky = host.querySelector('[data-vize-ui="sticky"]');
      assert.ok(sticky instanceof HTMLElement);
      assert.equal(sticky.getAttribute("data-state"), "idle");
    },
  },
];
