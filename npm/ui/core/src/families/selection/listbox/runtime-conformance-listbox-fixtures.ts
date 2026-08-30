import assert from "node:assert/strict";

import { h } from "vue";

import Listbox from "./listbox.vue";
import ListboxItem from "./listbox-item.vue";
import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";

function renderListboxFixture() {
  return h(
    Listbox,
    {
      ariaLabel: "Project status",
      defaultValue: ["todo", "done"],
      id: "status-listbox",
      selectionMode: "multiple",
    },
    {
      default: () => [
        h(ListboxItem, { textValue: "Todo", value: "todo" }, () => "Todo"),
        h(ListboxItem, { textValue: "Doing", value: "doing" }, () => "Doing"),
        h(ListboxItem, { textValue: "Done", value: "done" }, () => "Done"),
      ],
    },
  );
}

function assertListboxServerMarkup(html: string): void {
  assert.match(html, /^<div/);
  assert.match(html, /id="status-listbox"/);
  assert.match(html, /role="listbox"/);
  assert.match(html, /aria-label="Project status"/);
  assert.match(html, /aria-orientation="vertical"/);
  assert.match(html, /aria-multiselectable="true"/);
  assert.match(html, /data-vize-ui="listbox"/);
  assert.match(html, /data-state="selected"/);
  assert.match(html, /data-selection-mode="multiple"/);
  assert.match(html, /data-selection-count="2"/);
  assert.match(html, /role="option"/);
  assert.match(html, /data-vize-ui="listbox-item"/);
  assert.match(html, /data-value="todo"/);
  assert.match(html, /aria-selected="true"/);
  assert.match(html, /data-value="doing"/);
}

function assertListboxHydratedDom(host: HTMLElement): void {
  const root = host.querySelector('[data-vize-ui="listbox"]');
  const todo = host.querySelector<HTMLElement>('[data-vize-ui="listbox-item"][data-value="todo"]');
  const doing = host.querySelector<HTMLElement>(
    '[data-vize-ui="listbox-item"][data-value="doing"]',
  );
  const done = host.querySelector<HTMLElement>('[data-vize-ui="listbox-item"][data-value="done"]');

  assert.ok(root instanceof HTMLDivElement);
  assert.equal(root.id, "status-listbox");
  assert.equal(root.getAttribute("role"), "listbox");
  assert.equal(root.getAttribute("aria-multiselectable"), "true");
  assert.equal(root.getAttribute("data-selection-count"), "2");
  assert.ok(todo instanceof HTMLDivElement);
  assert.ok(doing instanceof HTMLDivElement);
  assert.ok(done instanceof HTMLDivElement);
  assert.equal(todo.getAttribute("aria-selected"), "true");
  assert.equal(doing.getAttribute("aria-selected"), "false");
  assert.equal(done.getAttribute("aria-selected"), "true");
}

export const listboxRuntimeFixtures: readonly RuntimeFixture[] = [
  {
    name: "listbox",
    sourceFile: "families/selection/listbox/listbox.vue",
    render: renderListboxFixture,
    assertServerMarkup: assertListboxServerMarkup,
    assertHydratedDom: assertListboxHydratedDom,
  },
  {
    name: "listbox-item",
    sourceFile: "families/selection/listbox/listbox-item.vue",
    render: renderListboxFixture,
    assertServerMarkup: assertListboxServerMarkup,
    assertHydratedDom: assertListboxHydratedDom,
  },
];
