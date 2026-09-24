import assert from "node:assert/strict";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { renderTransferListTree } from "./transfer-list-fixture-tree.ts";

function assertServerMarkup(html: string): void {
  assert.match(html, /^<div id="fruit-transfer"/);
  assert.match(html, /data-vize-ui="transfer-list"/);
  assert.match(html, /data-vize-ui="transfer-list-panel"/);
  assert.match(html, /data-vize-ui="transfer-list-item"/);
  assert.match(html, /data-vize-ui="transfer-list-search"/);
  assert.match(html, /data-vize-ui="transfer-list-action"/);
  assert.match(html, /data-vize-ui="transfer-list-empty"/);
  assert.match(html, /role="option"/);
}

function assertHydratedDom(host: HTMLElement): void {
  const panels = host.querySelectorAll('[data-vize-ui="transfer-list-panel"]');
  assert.equal(panels.length, 2);
  assert.equal(panels[1]?.querySelectorAll('[role="option"]').length, 1);
  const hidden = host.querySelector('[data-vize-ui="transfer-list-native"]');
  assert.ok(hidden instanceof HTMLInputElement);
  assert.equal(hidden.value, "Banana");
}

export const transferListRuntimeFixtures: readonly RuntimeFixture[] = [
  "transfer-list-action.vue",
  "transfer-list-empty.vue",
  "transfer-list-item.vue",
  "transfer-list-panel.vue",
  "transfer-list-root.vue",
  "transfer-list-search.vue",
].map((file) => ({
  name: file.replace(".vue", ""),
  sourceFile: `families/selection/transfer-list/${file}`,
  render: renderTransferListTree,
  assertServerMarkup,
  assertHydratedDom,
}));
