import assert from "node:assert/strict";

import { h } from "vue";

import BottomNavigation from "./bottom-navigation.vue";
import BottomNavigationItem from "./bottom-navigation-item.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const render = () =>
  h(
    BottomNavigation,
    { destinations: ["a", "b"], defaultValue: "a" },
    {
      default: () => [
        h(BottomNavigationItem, { value: "a" }, { default: () => "A" }),
        h(BottomNavigationItem, { value: "b" }, { default: () => "B" }),
      ],
    },
  );

export const bottomNavigationRuntimeFixtures: readonly RuntimeFixture[] = [
  "bottom-navigation.vue",
  "bottom-navigation-item.vue",
].map((file) => ({
  name: file.replace(/\.vue$/, ""),
  sourceFile: `families/navigation/bottom-navigation/${file}`,
  render,
  assertServerMarkup(html: string) {
    assert.match(html, /^<nav/);
    assert.match(html, /aria-current="page"/);
  },
  assertHydratedDom(host: HTMLElement) {
    assert.equal(host.querySelectorAll("button").length, 2);
  },
}));
