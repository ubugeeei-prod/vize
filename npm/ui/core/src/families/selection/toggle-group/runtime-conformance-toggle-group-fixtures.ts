import assert from "node:assert/strict";

import { h } from "vue";

import ToggleGroup from "./toggle-group.vue";
import ToggleGroupItem from "./toggle-group-item.vue";
import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";

function renderToggleGroupFixture() {
  return h(
    ToggleGroup,
    {
      ariaLabel: "Formatting",
      defaultValue: ["bold"],
      type: "multiple",
    },
    {
      default: () => [
        h(ToggleGroupItem, { value: "bold" }, () => "Bold"),
        h(ToggleGroupItem, { value: "italic" }, () => "Italic"),
      ],
    },
  );
}

function assertToggleGroupServerMarkup(html: string): void {
  assert.match(html, /^<div/);
  assert.match(html, /role="group"/);
  assert.match(html, /aria-label="Formatting"/);
  assert.match(html, /data-vize-ui="toggle-group"/);
  assert.match(html, /data-state="selected"/);
  assert.match(html, /data-type="multiple"/);
  assert.match(html, /data-vize-ui="toggle-group-item"/);
  assert.match(html, /aria-pressed="true"/);
  assert.match(html, /Bold/);
}

function assertToggleGroupHydratedDom(host: HTMLElement): void {
  const group = host.querySelector('[data-vize-ui="toggle-group"]');
  const bold = host.querySelector<HTMLButtonElement>(
    '[data-vize-ui="toggle-group-item"][data-value="bold"]',
  );
  const italic = host.querySelector<HTMLButtonElement>(
    '[data-vize-ui="toggle-group-item"][data-value="italic"]',
  );
  assert.ok(group instanceof HTMLElement);
  assert.ok(bold instanceof HTMLButtonElement);
  assert.ok(italic instanceof HTMLButtonElement);
  assert.equal(group.getAttribute("role"), "group");
  assert.equal(group.getAttribute("data-value"), "bold");
  assert.equal(bold.getAttribute("aria-pressed"), "true");
  assert.equal(italic.getAttribute("aria-pressed"), "false");
}

export const toggleGroupRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "toggle-group",
    sourceFile: "families/selection/toggle-group/toggle-group.vue",
    render: renderToggleGroupFixture,
    assertServerMarkup: assertToggleGroupServerMarkup,
    assertHydratedDom: assertToggleGroupHydratedDom,
  },
  {
    name: "toggle-group-item",
    sourceFile: "families/selection/toggle-group/toggle-group-item.vue",
    render: renderToggleGroupFixture,
    assertServerMarkup: assertToggleGroupServerMarkup,
    assertHydratedDom: assertToggleGroupHydratedDom,
  },
];
