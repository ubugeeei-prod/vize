import assert from "node:assert/strict";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { renderEmojiPickerTree } from "./emoji-picker-fixture-tree.ts";

function renderFixture() {
  return renderEmojiPickerTree();
}

function assertServerMarkup(html: string): void {
  assert.match(html, /^<div id="emoji"/);
  assert.match(html, /data-vize-ui="emoji-picker"/);
  assert.match(html, /data-vize-ui="emoji-picker-search"/);
  assert.match(html, /data-vize-ui="emoji-picker-skin-tone"/);
  assert.match(html, /data-vize-ui="emoji-picker-grid"/);
  assert.match(html, /data-vize-ui="emoji-picker-category"/);
  assert.match(html, /data-vize-ui="emoji-picker-item"/);
  assert.match(html, /data-vize-ui="emoji-picker-empty"/);
  assert.match(html, /data-vize-ui="emoji-picker-preview"/);
}

function assertHydratedDom(host: HTMLElement): void {
  const grid = host.querySelector('[data-vize-ui="emoji-picker-grid"]');
  assert.ok(grid instanceof HTMLDivElement);
  assert.equal(grid.getAttribute("role"), "grid");
  assert.equal(host.querySelectorAll('[role="gridcell"]').length, 6);
  assert.equal(host.querySelectorAll('[role="radio"]').length, 6);
}

export const emojiPickerRuntimeFixtures: readonly RuntimeFixture[] = [
  "emoji-picker-category.vue",
  "emoji-picker-empty.vue",
  "emoji-picker-grid.vue",
  "emoji-picker-item.vue",
  "emoji-picker-preview.vue",
  "emoji-picker-root.vue",
  "emoji-picker-search.vue",
  "emoji-picker-skin-tone.vue",
].map((file) => ({
  name: file.replace(".vue", ""),
  sourceFile: `families/selection/emoji-picker/${file}`,
  render: renderFixture,
  assertServerMarkup,
  assertHydratedDom,
}));
