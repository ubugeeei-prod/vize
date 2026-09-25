import assert from "node:assert/strict";

import { h } from "vue";

import SwipeActions from "./swipe-actions.vue";
import SwipeActionsAction from "./swipe-actions-action.vue";
import SwipeActionsContent from "./swipe-actions-content.vue";
import SwipeActionsTray from "./swipe-actions-tray.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const render = () =>
  h(SwipeActions, null, () => [
    h(SwipeActionsTray, { side: "trailing" }, () =>
      h(SwipeActionsAction, { value: "delete" }, () => "Delete"),
    ),
    h(SwipeActionsContent, null, () => "Row"),
  ]);

export const swipeActionsRuntimeFixtures: readonly RuntimeFixture[] = [
  "swipe-actions.vue",
  "swipe-actions-content.vue",
  "swipe-actions-tray.vue",
  "swipe-actions-action.vue",
].map((file) => ({
  name: file.replace(/\.vue$/, ""),
  sourceFile: `families/interaction/swipe-actions/${file}`,
  render,
  assertServerMarkup(html: string) {
    assert.match(html, /data-vize-ui="swipe-actions"/);
  },
  assertHydratedDom(host: HTMLElement) {
    assert.equal(host.querySelector('[data-side="trailing"]')?.hasAttribute("inert"), true);
  },
}));
