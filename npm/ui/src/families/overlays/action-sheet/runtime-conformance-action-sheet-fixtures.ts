import assert from "node:assert/strict";

import { h } from "vue";

import { ActionSheet, ActionSheetContent, ActionSheetTitle } from "./action-sheet.ts";
import ActionSheetItem from "./action-sheet-item.vue";
import ActionSheetMenu from "./action-sheet-menu.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const render = () =>
  h(ActionSheet, { id: "sheet", defaultOpen: true }, () =>
    h(ActionSheetContent, null, () => [
      h(ActionSheetTitle, null, () => "Share"),
      h(ActionSheetMenu, null, () => [h(ActionSheetItem, { value: "copy" }, () => "Copy")]),
    ]),
  );

export const actionSheetRuntimeFixtures: readonly RuntimeFixture[] = [
  "action-sheet-menu.vue",
  "action-sheet-item.vue",
].map((file) => ({
  name: file.replace(/\.vue$/, ""),
  sourceFile: `families/overlays/action-sheet/${file}`,
  render,
  assertServerMarkup(html: string) {
    assert.match(html, /role="menuitem"/);
  },
  assertHydratedDom(host: HTMLElement) {
    assert.equal(host.querySelector('[role="menuitem"]')?.textContent, "Copy");
  },
}));
