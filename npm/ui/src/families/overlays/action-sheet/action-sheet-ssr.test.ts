import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import { ActionSheet, ActionSheetContent, ActionSheetTitle } from "./action-sheet.ts";
import ActionSheetItem from "./action-sheet-item.vue";
import ActionSheetMenu from "./action-sheet-menu.vue";
import { renderAndHydrate } from "../../../testing/ssr-hydration.ts";

test("renders an open sheet's menu deterministically and hydrates without mismatches", async () => {
  const { html, dispose } = await renderAndHydrate(
    defineComponent({
      name: "ActionSheetSsrProbe",
      setup: () => () =>
        h(ActionSheet, { id: "sheet", defaultOpen: true }, () =>
          h(ActionSheetContent, null, () => [
            h(ActionSheetTitle, null, () => "Share"),
            h(ActionSheetMenu, null, () => [
              h(ActionSheetItem, { value: "copy" }, () => "Copy"),
              h(ActionSheetItem, { value: "mail" }, () => "Mail"),
            ]),
          ]),
        ),
    }),
  );
  try {
    assert.match(html, /role="menu"/);
    assert.match(html, /role="menuitem" tabindex="0"[^>]*data-value="copy"/);
    assert.match(html, /role="menuitem" tabindex="-1"[^>]*data-value="mail"/);
  } finally {
    dispose();
  }
});
