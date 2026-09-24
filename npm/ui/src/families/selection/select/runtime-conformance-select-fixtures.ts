import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import SelectContent from "./select-content.vue";
import SelectGroup from "./select-group.vue";
import SelectItem from "./select-item.vue";
import SelectItemIndicator from "./select-item-indicator.vue";
import SelectLabel from "./select-label.vue";
import SelectRoot from "./select-root.vue";
import SelectScrollButton from "./select-scroll-button.vue";
import SelectSeparator from "./select-separator.vue";
import SelectTrigger from "./select-trigger.vue";
import SelectValue from "./select-value.vue";
import SelectViewport from "./select-viewport.vue";
import SelectVirtualizer from "./select-virtualizer.vue";

const statuses = ["todo", "doing", "done"] as const;
type Status = (typeof statuses)[number];

function renderSelectFixture() {
  return h(
    SelectRoot<Status>,
    {
      defaultOpen: true,
      defaultValue: "doing",
      id: "status-select",
      items: statuses,
      name: "status",
    },
    () => [
      h(SelectTrigger, { ariaLabel: "Status" }, () => h(SelectValue)),
      h(SelectContent, null, () => [
        h(SelectScrollButton, { direction: "up" }, () => "Up"),
        h(SelectViewport, null, () => [
          h(SelectGroup, null, () => [
            h(SelectLabel, null, () => "Workflow"),
            h(SelectItem<Status>, { index: 0, value: "todo" }, () => "Todo"),
            h(SelectItem<Status>, { index: 1, value: "doing" }, () => [
              "Doing",
              h(SelectItemIndicator, null, () => "Selected"),
            ]),
          ]),
          h(SelectSeparator),
          h(
            SelectVirtualizer<Status>,
            { estimateItemSize: 20, initialViewportHeight: 20, items: ["done"], overscan: 0 },
            {
              default: ({ index, item }: { readonly index: number; readonly item: Status }) =>
                h(SelectItem<Status>, { index: index + 2, value: item }, () => "Done"),
            },
          ),
        ]),
        h(SelectScrollButton, { direction: "down" }, () => "Down"),
      ]),
    ],
  );
}

function assertSelectServerMarkup(html: string): void {
  assert.match(html, /^<div id="status-select"/);
  assert.match(html, /data-vize-ui="select"/);
  assert.match(html, /role="combobox"/);
  assert.match(html, /aria-expanded="true"/);
  assert.match(html, /aria-controls="status-select-listbox"/);
  assert.match(html, /id="status-select-listbox"/);
  assert.match(html, /role="listbox"/);
  assert.match(html, /role="group"/);
  assert.match(html, /data-vize-ui="select-label"/);
  assert.match(html, /data-vize-ui="select-viewport"/);
  assert.match(html, /data-vize-ui="select-virtualizer"/);
  assert.match(html, /data-vize-ui="select-separator"/);
  assert.match(html, /data-vize-ui="select-item-indicator"/);
  assert.match(html, /role="option"[^>]*aria-selected="true"/);
  assert.match(html, /data-vize-ui="select-native"/);
  assert.match(html, /name="status"/);
}

function assertSelectHydratedDom(host: HTMLElement): void {
  const root = host.querySelector('[data-vize-ui="select"]');
  const trigger = host.querySelector('[data-vize-ui="select-trigger"]');
  const native = host.querySelector('[data-vize-ui="select-native"]');
  assert.ok(root instanceof HTMLDivElement);
  assert.ok(trigger instanceof HTMLButtonElement);
  assert.ok(native instanceof HTMLSelectElement);
  assert.equal(trigger.getAttribute("role"), "combobox");
  assert.equal(trigger.getAttribute("aria-expanded"), "true");
  assert.equal(native.value, "doing");
  const options = [...document.querySelectorAll('[data-vize-ui="select-item"]')];
  assert.ok(options.length >= 3);
}

const selectSourceFiles = [
  "select-content.vue",
  "select-group.vue",
  "select-item-indicator.vue",
  "select-item.vue",
  "select-label.vue",
  "select-root.vue",
  "select-scroll-button.vue",
  "select-separator.vue",
  "select-trigger.vue",
  "select-value.vue",
  "select-viewport.vue",
  "select-virtualizer.vue",
] as const;

export const selectRuntimeFixtures: readonly RuntimeFixture[] = selectSourceFiles.map((file) => ({
  name: file.replace(".vue", ""),
  sourceFile: `families/selection/select/${file}`,
  render: renderSelectFixture,
  assertServerMarkup: assertSelectServerMarkup,
  assertHydratedDom: assertSelectHydratedDom,
}));
