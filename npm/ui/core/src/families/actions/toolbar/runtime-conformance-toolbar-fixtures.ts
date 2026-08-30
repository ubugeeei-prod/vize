import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";
import { Toolbar, ToolbarItem } from "./toolbar.ts";

function renderToolbarFixture() {
  return h(
    Toolbar,
    {
      ariaLabel: "Editor actions",
      dir: "rtl",
    },
    {
      default: () => [
        h(ToolbarItem, { value: "save" }, () => "Save"),
        h(ToolbarItem, { value: "publish" }, () => "Publish"),
      ],
    },
  );
}

function assertToolbarServerMarkup(html: string): void {
  assert.match(html, /^<div/);
  assert.match(html, /role="toolbar"/);
  assert.match(html, /aria-label="Editor actions"/);
  assert.match(html, /aria-orientation="horizontal"/);
  assert.match(html, /dir="rtl"/);
  assert.match(html, /data-vize-ui="toolbar"/);
  assert.match(html, /data-state="idle"/);
  assert.match(html, /data-roving-focus="true"/);
  assert.match(html, /--vize-ui-toolbar-orientation:horizontal/);
  assert.match(html, /data-vize-ui="toolbar-item"/);
  assert.match(html, /data-value="save"/);
  assert.match(html, /Save/);
}

function assertToolbarHydratedDom(host: HTMLElement): void {
  const toolbar = host.querySelector('[data-vize-ui="toolbar"]');
  const save = host.querySelector<HTMLButtonElement>(
    '[data-vize-ui="toolbar-item"][data-value="save"]',
  );
  const publish = host.querySelector<HTMLButtonElement>(
    '[data-vize-ui="toolbar-item"][data-value="publish"]',
  );
  assert.ok(toolbar instanceof HTMLElement);
  assert.ok(save instanceof HTMLButtonElement);
  assert.ok(publish instanceof HTMLButtonElement);
  assert.equal(toolbar.getAttribute("role"), "toolbar");
  assert.equal(toolbar.getAttribute("dir"), "rtl");
  assert.equal(toolbar.getAttribute("data-roving-focus"), "true");
  assert.equal(save.type, "button");
  assert.equal(save.getAttribute("tabindex"), "0");
  assert.equal(publish.getAttribute("tabindex"), "-1");
}

export const toolbarRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "toolbar",
    sourceFile: "families/actions/toolbar/toolbar.vue",
    render: renderToolbarFixture,
    assertServerMarkup: assertToolbarServerMarkup,
    assertHydratedDom: assertToolbarHydratedDom,
  },
  {
    name: "toolbar-item",
    sourceFile: "families/actions/toolbar/toolbar-item.vue",
    render: renderToolbarFixture,
    assertServerMarkup: assertToolbarServerMarkup,
    assertHydratedDom: assertToolbarHydratedDom,
  },
];
