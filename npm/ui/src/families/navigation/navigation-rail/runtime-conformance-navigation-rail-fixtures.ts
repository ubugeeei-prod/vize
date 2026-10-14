import assert from "node:assert/strict";

import { h } from "vue";

import NavigationRail from "./navigation-rail.vue";
import NavigationRailItem from "./navigation-rail-item.vue";
import NavigationRailToggle from "./navigation-rail-toggle.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const render = () =>
  h(
    NavigationRail,
    { destinations: ["a", "b"], defaultValue: "a", id: "runtime-rail" },
    {
      default: () => [
        h(NavigationRailToggle, { ariaLabel: "Toggle navigation" }),
        h(NavigationRailItem, { value: "a" }, { default: () => "A" }),
        h(NavigationRailItem, { value: "b" }, { default: () => "B" }),
      ],
    },
  );

export const navigationRailRuntimeFixtures: readonly RuntimeFixture[] = [
  "navigation-rail.vue",
  "navigation-rail-item.vue",
  "navigation-rail-toggle.vue",
].map((file) => ({
  name: file.replace(/\.vue$/, ""),
  sourceFile: `families/navigation/navigation-rail/${file}`,
  render,
  assertServerMarkup(html: string) {
    assert.match(html, /^<nav id="runtime-rail"/);
    assert.match(html, /aria-current="page"/);
    assert.match(html, /aria-expanded="false" aria-controls="runtime-rail"/);
  },
  assertHydratedDom(host: HTMLElement) {
    assert.equal(host.querySelectorAll('[data-vize-ui="navigation-rail-item"]').length, 2);
    assert.equal(
      host.querySelector('[data-vize-ui="navigation-rail-toggle"]')?.getAttribute("aria-controls"),
      host.querySelector("nav")?.id,
    );
  },
}));
