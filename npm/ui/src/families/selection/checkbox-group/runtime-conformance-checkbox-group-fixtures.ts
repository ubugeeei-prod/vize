import assert from "node:assert/strict";

import { h } from "vue";

import CheckboxGroup from "./checkbox-group.vue";
import CheckboxGroupItem from "./checkbox-group-item.vue";
import CheckboxGroupSelectAll from "./checkbox-group-select-all.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const options = ["email", "sms"] as const;

const render = () =>
  h(
    CheckboxGroup,
    { options, defaultValue: ["sms"], name: "channels", ariaLabel: "Channels" },
    {
      default: () => [
        h(CheckboxGroupSelectAll, { ariaLabel: "All channels" }),
        ...options.map((option) =>
          h(CheckboxGroupItem, { key: option, value: option, ariaLabel: option }),
        ),
      ],
    },
  );

function assertServerMarkup(html: string): void {
  assert.match(html, /^<div role="group"/);
  assert.match(html, /data-vize-ui="checkbox-group"/);
  assert.match(html, /data-vize-ui="checkbox-group-select-all"/);
  assert.match(html, /value="sms" checked/);
}

function assertHydratedDom(host: HTMLElement): void {
  const items = host.querySelectorAll<HTMLInputElement>('[data-vize-ui="checkbox-group-item"]');
  assert.equal(items.length, 2);
  assert.equal(items[1]?.checked, true);
  const parent = host.querySelector<HTMLInputElement>('[data-vize-ui="checkbox-group-select-all"]');
  assert.equal(parent?.getAttribute("aria-checked"), "mixed");
}

export const checkboxGroupRuntimeFixtures: readonly RuntimeFixture[] = [
  "checkbox-group.vue",
  "checkbox-group-item.vue",
  "checkbox-group-select-all.vue",
].map((file) => ({
  name: file.replace(/\.vue$/, ""),
  sourceFile: `families/selection/checkbox-group/${file}`,
  render,
  assertServerMarkup,
  assertHydratedDom,
}));
