import assert from "node:assert/strict";

import { h } from "vue";

import NativeSelect from "./native-select.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

function renderNativeSelectFixture() {
  return h(NativeSelect, {
    ariaLabel: "Project status",
    defaultValue: ["todo", "done"],
    id: "status-native-select",
    multiple: true,
    name: "status",
    options: [
      { label: "Todo", value: "todo" },
      { label: "Doing", value: "doing" },
      { label: "Done", value: "done" },
    ],
    size: 3,
  });
}

function assertNativeSelectServerMarkup(html: string): void {
  assert.match(html, /^<select/);
  assert.match(html, /id="status-native-select"/);
  assert.match(html, /name="status"/);
  assert.match(html, /multiple/);
  assert.match(html, /size="3"/);
  assert.match(html, /aria-label="Project status"/);
  assert.match(html, /data-vize-ui="native-select"/);
  assert.match(html, /data-state="selected"/);
  assert.match(html, /data-selection-mode="multiple"/);
  assert.match(html, /data-selection-count="2"/);
  assert.match(html, /data-direction="ltr"/);
  assert.match(html, /data-vize-ui="native-select-option"/);
  assert.match(html, /data-value="todo"/);
  assert.match(html, /data-selected="true"/);
  assert.match(html, /data-value="doing"/);
}

function assertNativeSelectHydratedDom(host: HTMLElement): void {
  const select = host.querySelector('[data-vize-ui="native-select"]');
  const todo = host.querySelector<HTMLOptionElement>(
    '[data-vize-ui="native-select-option"][data-value="todo"]',
  );
  const doing = host.querySelector<HTMLOptionElement>(
    '[data-vize-ui="native-select-option"][data-value="doing"]',
  );
  const done = host.querySelector<HTMLOptionElement>(
    '[data-vize-ui="native-select-option"][data-value="done"]',
  );

  assert.ok(select instanceof HTMLSelectElement);
  assert.equal(select.id, "status-native-select");
  assert.equal(select.name, "status");
  assert.equal(select.multiple, true);
  assert.equal(select.getAttribute("data-selection-count"), "2");
  assert.ok(todo instanceof HTMLOptionElement);
  assert.ok(doing instanceof HTMLOptionElement);
  assert.ok(done instanceof HTMLOptionElement);
  assert.equal(todo.selected, true);
  assert.equal(doing.selected, false);
  assert.equal(done.selected, true);
}

export const nativeSelectRuntimeFixture: RuntimeFixture = {
  name: "native-select",
  sourceFile: "families/selection/native-select/native-select.vue",
  render: renderNativeSelectFixture,
  assertServerMarkup: assertNativeSelectServerMarkup,
  assertHydratedDom: assertNativeSelectHydratedDom,
};
