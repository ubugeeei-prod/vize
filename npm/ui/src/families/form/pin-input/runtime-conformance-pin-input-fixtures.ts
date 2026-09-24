import assert from "node:assert/strict";

import { h } from "vue";

import PinInput from "./pin-input.vue";
import PinInputField from "./pin-input-field.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const render = () =>
  h(
    PinInput,
    { length: 4, id: "otp", name: "otp", defaultValue: "12", ariaLabel: "Code" },
    { default: () => [0, 1, 2, 3].map((index) => h(PinInputField, { key: index, index })) },
  );

function assertServerMarkup(html: string): void {
  assert.match(html, /^<div role="group"/);
  assert.match(html, /id="otp-0"/);
  assert.match(html, /autocomplete="one-time-code"/);
}

function assertHydratedDom(host: HTMLElement): void {
  const fields = host.querySelectorAll<HTMLInputElement>('[data-vize-ui="pin-input-field"]');
  assert.equal(fields.length, 4);
  assert.equal(fields[1]?.value, "2");
  assert.equal(host.querySelector<HTMLInputElement>('input[type="hidden"]')?.value, "12");
}

export const pinInputRuntimeFixtures: readonly RuntimeFixture[] = [
  "pin-input.vue",
  "pin-input-field.vue",
].map((file) => ({
  name: file.replace(/\.vue$/, ""),
  sourceFile: `families/form/pin-input/${file}`,
  render,
  assertServerMarkup,
  assertHydratedDom,
}));
