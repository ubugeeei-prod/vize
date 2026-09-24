import assert from "node:assert/strict";

import { h } from "vue";

import { DirectionProvider } from "./direction.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const directionRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "direction-provider",
    sourceFile: "families/i18n/direction/direction-provider.vue",
    render: () => h(DirectionProvider, { dir: "rtl" }, () => "مرحبا"),
    assertServerMarkup(html) {
      assert.match(html, /dir="rtl"/);
      assert.match(html, /data-vize-ui="direction-provider"/);
    },
    assertHydratedDom(host) {
      const root = host.querySelector('[data-vize-ui="direction-provider"]');
      assert.ok(root instanceof HTMLElement);
      assert.equal(root.dir, "rtl");
    },
  },
];
