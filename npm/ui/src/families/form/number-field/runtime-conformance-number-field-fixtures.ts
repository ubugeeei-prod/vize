import assert from "node:assert/strict";

import { h } from "vue";

import NumberField from "./number-field.vue";
import NumberFieldDecrement from "./number-field-decrement.vue";
import NumberFieldIncrement from "./number-field-increment.vue";
import NumberFieldInput from "./number-field-input.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const render = () =>
  h(
    NumberField,
    {
      ariaLabel: "Quantity",
      defaultValue: 3,
      id: "quantity",
      max: 10,
      min: 0,
      name: "quantity",
    },
    {
      default: () => [h(NumberFieldDecrement), h(NumberFieldInput), h(NumberFieldIncrement)],
    },
  );

function assertServerMarkup(html: string): void {
  assert.match(html, /^<div/);
  assert.match(html, /data-vize-ui="number-field"/);
  assert.match(html, /type="hidden" name="quantity" value="3"/);
  assert.match(html, /role="spinbutton"/);
  assert.match(html, /aria-valuenow="3"/);
  assert.match(html, /aria-controls="quantity"/);
}

function assertHydratedDom(host: HTMLElement): void {
  const input = host.querySelector('[data-vize-ui="number-field-input"]');
  const increment = host.querySelector('[data-vize-ui="number-field-increment"]');
  const decrement = host.querySelector('[data-vize-ui="number-field-decrement"]');
  assert.ok(input instanceof HTMLInputElement);
  assert.ok(increment instanceof HTMLButtonElement);
  assert.ok(decrement instanceof HTMLButtonElement);
  assert.equal(input.id, "quantity");
  assert.equal(input.value, "3");
  assert.equal(input.getAttribute("role"), "spinbutton");
  assert.equal(increment.getAttribute("aria-controls"), "quantity");
  assert.equal(decrement.getAttribute("aria-label"), "Decrease");
}

export const numberFieldRuntimeFixtures: readonly RuntimeFixture[] = [
  "number-field.vue",
  "number-field-input.vue",
  "number-field-increment.vue",
  "number-field-decrement.vue",
].map((file) => ({
  name: file.replace(/\.vue$/, ""),
  sourceFile: `families/form/number-field/${file}`,
  render,
  assertServerMarkup,
  assertHydratedDom,
}));
