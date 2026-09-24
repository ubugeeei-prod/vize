import assert from "node:assert/strict";

import { h } from "vue";

import PasswordField from "./password-field.vue";
import PasswordFieldInput from "./password-field-input.vue";
import PasswordFieldToggle from "./password-field-toggle.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const render = () =>
  h(
    PasswordField,
    { id: "password", name: "password", ariaLabel: "Password", defaultValue: "secret" },
    { default: () => [h(PasswordFieldInput), h(PasswordFieldToggle)] },
  );

function assertServerMarkup(html: string): void {
  assert.match(html, /^<div part="root" data-vize-ui="password-field"/);
  assert.match(html, /type="password"/);
  assert.match(html, /aria-controls="password"/);
}

function assertHydratedDom(host: HTMLElement): void {
  const input = host.querySelector("input");
  assert.ok(input instanceof HTMLInputElement);
  assert.equal(input.value, "secret");
  assert.equal(host.querySelector("button")?.getAttribute("aria-pressed"), "false");
}

export const passwordFieldRuntimeFixtures: readonly RuntimeFixture[] = [
  "password-field.vue",
  "password-field-input.vue",
  "password-field-toggle.vue",
].map((file) => ({
  name: file.replace(/\.vue$/, ""),
  sourceFile: `families/form/password-field/${file}`,
  render,
  assertServerMarkup,
  assertHydratedDom,
}));
