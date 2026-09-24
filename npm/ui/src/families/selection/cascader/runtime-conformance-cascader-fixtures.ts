import assert from "node:assert/strict";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import { places, renderCascaderTree } from "./cascader-fixture-tree.ts";

const japan = places[0];
const osaka = japan?.children?.[1];
const kita = osaka?.children?.[0];

function renderFixture() {
  return renderCascaderTree(
    {
      defaultOpen: true,
      defaultValue: [japan, osaka, kita],
      id: "runtime-cascader",
      name: "place",
    },
    { portalDisabled: true },
  );
}

function assertServerMarkup(html: string): void {
  assert.match(html, /^<div id="runtime-cascader"/);
  assert.match(html, /data-vize-ui="cascader-trigger"/);
  assert.match(html, /data-vize-ui="cascader-value"/);
  assert.match(html, /data-vize-ui="cascader-content"/);
  assert.match(html, /data-vize-ui="cascader-column"/);
  assert.match(html, /data-vize-ui="cascader-item"/);
  assert.match(html, /Japan \/ Osaka \/ Kita/);
}

function assertHydratedDom(host: HTMLElement): void {
  const trigger = host.querySelector('[data-vize-ui="cascader-trigger"]');
  assert.ok(trigger instanceof HTMLButtonElement);
  assert.equal(trigger.getAttribute("aria-expanded"), "true");
  assert.equal(host.querySelectorAll('[data-vize-ui="cascader-column"]').length, 3);
  const hidden = host.querySelector('[data-vize-ui="cascader-native"]');
  assert.ok(hidden instanceof HTMLInputElement);
  assert.equal(hidden.value, "jp / osaka / kita");
}

export const cascaderRuntimeFixtures: readonly RuntimeFixture[] = [
  "cascader-column.vue",
  "cascader-content.vue",
  "cascader-item.vue",
  "cascader-root.vue",
  "cascader-trigger.vue",
  "cascader-value.vue",
].map((file) => ({
  name: file.replace(".vue", ""),
  sourceFile: `families/selection/cascader/${file}`,
  render: renderFixture,
  assertServerMarkup,
  assertHydratedDom,
}));
