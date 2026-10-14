import assert from "node:assert/strict";

import { h } from "vue";

import MenuContent from "../../menus/menu/menu-content.vue";
import MenuItem from "../../menus/menu/menu-item.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import SplitButtonAction from "./split-button-action.vue";
import SplitButtonRoot from "./split-button-root.vue";
import SplitButtonTrigger from "./split-button-trigger.vue";

function renderSplitButtonFixture() {
  return h("div", [
    h(
      SplitButtonRoot,
      { id: "runtime-split-button", ariaLabel: "Save options", defaultOpen: true },
      () => [
        h(SplitButtonAction, null, () => "Save"),
        h(SplitButtonTrigger, { ariaLabel: "More save options" }),
        h(MenuContent, { portalDisabled: true }, () => h(MenuItem, null, () => "Save as draft")),
      ],
    ),
  ]);
}

function assertSplitButtonServerMarkup(html: string): void {
  assert.match(html, /role="group"/);
  assert.match(html, /aria-label="Save options"/);
  assert.match(html, /data-vize-ui="split-button"/);
  assert.match(html, /data-vize-ui="split-button-action"/);
  assert.match(html, /data-vize-ui="split-button-trigger"/);
  assert.match(html, /aria-haspopup="menu"/);
  assert.match(html, /aria-expanded="true"/);
  assert.match(html, /data-menu-kind="split-button"/);
  assert.match(html, /aria-labelledby="runtime-split-button-trigger"/);
}

function assertSplitButtonHydratedDom(host: HTMLElement): void {
  const group = host.querySelector('[data-vize-ui="split-button"]');
  const action = host.querySelector('[data-vize-ui="split-button-action"]');
  const trigger = host.querySelector('[data-vize-ui="split-button-trigger"]');
  const content = host.querySelector('[data-vize-ui="menu-content"]');
  assert.ok(group instanceof HTMLElement);
  assert.ok(action instanceof HTMLButtonElement);
  assert.ok(trigger instanceof HTMLButtonElement);
  assert.ok(content instanceof HTMLElement);
  assert.equal(group.getAttribute("data-state"), "open");
  assert.equal(action.type, "button");
  assert.equal(action.textContent, "Save");
  assert.equal(trigger.id, "runtime-split-button-trigger");
  assert.equal(trigger.getAttribute("aria-controls"), content.id);
  assert.equal(content.id, "runtime-split-button-content");
}

export const splitButtonRuntimeFixtures: readonly RuntimeFixture[] = [
  "split-button-root",
  "split-button-action",
  "split-button-trigger",
].map((name) => ({
  name,
  sourceFile: `families/actions/split-button/${name}.vue`,
  render: renderSplitButtonFixture,
  assertServerMarkup: assertSplitButtonServerMarkup,
  assertHydratedDom: assertSplitButtonHydratedDom,
}));
