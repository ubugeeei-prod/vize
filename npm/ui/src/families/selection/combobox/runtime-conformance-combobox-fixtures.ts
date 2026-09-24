import assert from "node:assert/strict";

import { h } from "vue";

import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";
import SelectContent from "../select/select-content.vue";
import SelectItem from "../select/select-item.vue";
import ComboboxAnchor from "./combobox-anchor.vue";
import ComboboxChip from "./combobox-chip.vue";
import ComboboxChipRemove from "./combobox-chip-remove.vue";
import ComboboxCreateItem from "./combobox-create-item.vue";
import ComboboxEmpty from "./combobox-empty.vue";
import ComboboxInput from "./combobox-input.vue";
import ComboboxLoading from "./combobox-loading.vue";
import ComboboxRoot from "./combobox-root.vue";
import ComboboxTrigger from "./combobox-trigger.vue";
import type { ComboboxSlotState } from "./combobox-types.ts";

const tags: readonly string[] = ["bug", "docs", "perf"];

function renderComboboxFixture() {
  return h(
    ComboboxRoot<string, true>,
    {
      createOption: (text: string) => text,
      defaultOpen: true,
      defaultValue: ["docs"],
      id: "labels-combobox",
      items: tags,
      multiple: true,
      name: "labels",
    },
    {
      default: (state: ComboboxSlotState<string>) => [
        h(ComboboxAnchor, null, () => [
          ...state.selected.map((tag) =>
            h(ComboboxChip<string>, { key: tag, value: tag }, () => [
              tag,
              h(ComboboxChipRemove, null, () => "x"),
            ]),
          ),
          h(ComboboxInput, { ariaLabel: "Labels" }),
          h(ComboboxTrigger, null, () => "v"),
        ]),
        h(SelectContent, null, () => [
          h(ComboboxLoading, null, () => "Loading"),
          ...state.filteredItems.map((tag) =>
            h(SelectItem<string>, { key: tag, value: tag }, () => tag),
          ),
          h(ComboboxCreateItem, null, () => "Create"),
          h(ComboboxEmpty, null, () => "No labels"),
        ]),
      ],
    },
  );
}

function assertComboboxServerMarkup(html: string): void {
  assert.match(html, /^<div id="labels-combobox"/);
  assert.match(html, /data-vize-ui="combobox"/);
  assert.match(html, /data-vize-ui="combobox-anchor"/);
  assert.match(html, /role="combobox"/);
  assert.match(html, /aria-expanded="true"/);
  assert.match(html, /aria-controls="labels-combobox-listbox"/);
  assert.match(html, /data-vize-ui="combobox-chip"/);
  assert.match(html, /data-vize-ui="combobox-chip-remove"/);
  assert.match(html, /data-vize-ui="combobox-trigger"/);
  assert.match(html, /role="listbox"/);
  assert.match(html, /data-vize-ui="combobox-item"/);
  assert.match(html, /data-vize-ui="combobox-loading"/);
  assert.match(html, /data-vize-ui="combobox-create-item"/);
  assert.match(html, /data-vize-ui="combobox-empty"/);
  assert.match(html, /type="hidden"[^>]*name="labels"[^>]*value="docs"/);
}

function assertComboboxHydratedDom(host: HTMLElement): void {
  const input = host.querySelector('[data-vize-ui="combobox-input"]');
  assert.ok(input instanceof HTMLInputElement);
  assert.equal(input.getAttribute("role"), "combobox");
  assert.equal(input.getAttribute("aria-expanded"), "true");
  assert.equal(host.querySelectorAll('[data-vize-ui="combobox-chip"]').length, 1);
  const hidden = host.querySelector('[data-vize-ui="combobox-native"]');
  assert.ok(hidden instanceof HTMLInputElement);
  assert.equal(hidden.value, "docs");
}

const comboboxSourceFiles = [
  "combobox-anchor.vue",
  "combobox-chip-remove.vue",
  "combobox-chip.vue",
  "combobox-create-item.vue",
  "combobox-empty.vue",
  "combobox-input.vue",
  "combobox-loading.vue",
  "combobox-root.vue",
  "combobox-trigger.vue",
] as const;

export const comboboxRuntimeFixtures: readonly RuntimeFixture[] = comboboxSourceFiles.map(
  (file) => ({
    name: file.replace(".vue", ""),
    sourceFile: `families/selection/combobox/${file}`,
    render: renderComboboxFixture,
    assertServerMarkup: assertComboboxServerMarkup,
    assertHydratedDom: assertComboboxHydratedDom,
  }),
);
