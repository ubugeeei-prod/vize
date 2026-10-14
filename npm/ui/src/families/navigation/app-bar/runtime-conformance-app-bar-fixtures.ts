import assert from "node:assert/strict";

import { h } from "vue";

import AppBar from "./app-bar.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

export const appBarRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "app-bar",
    sourceFile: "families/navigation/app-bar/app-bar.vue",
    render: () =>
      h(
        AppBar,
        { ariaLabel: "Application", hideOnScroll: true },
        {
          start: () => h("button", { type: "button" }, "Menu"),
          default: () => "Inbox",
          end: () => h("button", { type: "button" }, "Search"),
        },
      ),
    assertServerMarkup(html: string) {
      assert.match(html, /^<header aria-label="Application"/);
      assert.match(html, /data-state="visible"/);
    },
    assertHydratedDom(host: HTMLElement) {
      assert.equal(host.querySelectorAll("header button").length, 2);
    },
  },
];
