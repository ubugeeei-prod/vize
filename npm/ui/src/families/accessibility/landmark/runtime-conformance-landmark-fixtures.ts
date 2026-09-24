import assert from "node:assert/strict";

import { h } from "vue";

import { Landmark, LandmarkProvider } from "./landmark.ts";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const landmarkRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "landmark",
    sourceFile: "families/accessibility/landmark/landmark.vue",
    render: () => h(Landmark, { role: "navigation", ariaLabel: "Primary" }, () => "Nav"),
    assertServerMarkup(html) {
      assert.match(html, /<nav[^>]*data-vize-ui="landmark"/);
      assert.match(html, /aria-label="Primary"/);
    },
    assertHydratedDom(host) {
      const nav = host.querySelector('[data-vize-ui="landmark"]');
      assert.ok(nav instanceof HTMLElement);
      assert.equal(nav.getAttribute("data-landmark"), "navigation");
    },
  },
  {
    name: "landmark-provider",
    sourceFile: "families/accessibility/landmark/landmark-provider.vue",
    render: () =>
      h(
        "div",
        null,
        h(LandmarkProvider, null, () => h(Landmark, { role: "main" }, () => "Main")),
      ),
    assertServerMarkup(html) {
      assert.match(html, /<main[^>]*data-landmark="main"/);
    },
    assertHydratedDom(host) {
      assert.ok(host.querySelector("main") instanceof HTMLElement);
    },
  },
];
