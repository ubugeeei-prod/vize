import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import SwipeActions from "./swipe-actions.vue";
import SwipeActionsAction from "./swipe-actions-action.vue";
import SwipeActionsContent from "./swipe-actions-content.vue";
import SwipeActionsTray from "./swipe-actions-tray.vue";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

test("renders byte-identical closed rows and hydrates without mismatches", async () => {
  const { html, dispose } = await renderAndHydrate(
    defineComponent({
      name: "SwipeActionsSsrProbe",
      setup: () => () =>
        h(SwipeActions, { as: "li" }, () => [
          h(SwipeActionsTray, { side: "trailing" }, () =>
            h(SwipeActionsAction, { value: "delete" }, () => "Delete"),
          ),
          h(SwipeActionsContent, null, () => "Inbox item"),
        ]),
    }),
  );
  try {
    assert.match(html, /^<li part="root" data-vize-ui="swipe-actions" data-state="closed"/);
    assert.match(html, /role="group" inert/);
    assert.match(html, /--vize-swipe-offset:0px/);
    assert.match(html, /aria-keyshortcuts="ArrowLeft ArrowRight Escape"/);
  } finally {
    dispose();
  }
});
