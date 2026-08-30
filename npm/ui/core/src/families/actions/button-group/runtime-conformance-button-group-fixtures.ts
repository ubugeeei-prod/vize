import assert from "node:assert/strict";

import { h } from "vue";

import ButtonGroup from "./button-group.vue";
import ButtonGroupItem from "./button-group-item.vue";
import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";

function renderButtonGroupFixture() {
  return h(
    ButtonGroup,
    {
      ariaLabel: "Editor actions",
      role: "toolbar",
    },
    {
      default: () => [
        h(ButtonGroupItem, { value: "save" }, () => "Save"),
        h(ButtonGroupItem, { value: "publish" }, () => "Publish"),
      ],
    },
  );
}

function assertButtonGroupServerMarkup(html: string): void {
  assert.match(html, /^<div/);
  assert.match(html, /role="toolbar"/);
  assert.match(html, /aria-label="Editor actions"/);
  assert.match(html, /aria-orientation="horizontal"/);
  assert.match(html, /data-vize-ui="button-group"/);
  assert.match(html, /data-state="idle"/);
  assert.match(html, /data-role="toolbar"/);
  assert.match(html, /data-vize-ui="button-group-item"/);
  assert.match(html, /data-value="save"/);
  assert.match(html, /Save/);
}

function assertButtonGroupHydratedDom(host: HTMLElement): void {
  const group = host.querySelector('[data-vize-ui="button-group"]');
  const save = host.querySelector<HTMLButtonElement>(
    '[data-vize-ui="button-group-item"][data-value="save"]',
  );
  const publish = host.querySelector<HTMLButtonElement>(
    '[data-vize-ui="button-group-item"][data-value="publish"]',
  );
  assert.ok(group instanceof HTMLElement);
  assert.ok(save instanceof HTMLButtonElement);
  assert.ok(publish instanceof HTMLButtonElement);
  assert.equal(group.getAttribute("role"), "toolbar");
  assert.equal(group.getAttribute("data-roving-focus"), "true");
  assert.equal(save.type, "button");
  assert.equal(save.getAttribute("tabindex"), "0");
  assert.equal(publish.getAttribute("tabindex"), "-1");
}

export const buttonGroupRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "button-group",
    sourceFile: "families/actions/button-group/button-group.vue",
    render: renderButtonGroupFixture,
    assertServerMarkup: assertButtonGroupServerMarkup,
    assertHydratedDom: assertButtonGroupHydratedDom,
  },
  {
    name: "button-group-item",
    sourceFile: "families/actions/button-group/button-group-item.vue",
    render: renderButtonGroupFixture,
    assertServerMarkup: assertButtonGroupServerMarkup,
    assertHydratedDom: assertButtonGroupHydratedDom,
  },
];
