import assert from "node:assert/strict";

import { h } from "vue";

import PhoneField from "./phone-field.vue";
import PhoneFieldCountrySelect from "./phone-field-country-select.vue";
import PhoneFieldInput from "./phone-field-input.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const countries = [
  { code: "JP", name: "Japan", dialCode: "81", pattern: "99-9999-9999", trunkPrefix: "0" },
  { code: "US", name: "United States", dialCode: "1", pattern: "(999) 999-9999" },
] as const;

const render = () =>
  h(
    PhoneField,
    { countries, id: "tel", defaultValue: "+819012345678", ariaLabel: "Phone" },
    { default: () => [h(PhoneFieldCountrySelect), h(PhoneFieldInput)] },
  );

function assertServerMarkup(html: string): void {
  assert.match(html, /data-vize-ui="phone-field"/);
  assert.match(html, /value="90-1234-5678"/);
}

function assertHydratedDom(host: HTMLElement): void {
  assert.equal(host.querySelector<HTMLInputElement>("#tel")?.value, "90-1234-5678");
  assert.equal(host.querySelector("select")?.value, "JP");
}

export const phoneFieldRuntimeFixtures: readonly RuntimeFixture[] = [
  "phone-field.vue",
  "phone-field-input.vue",
  "phone-field-country-select.vue",
].map((file) => ({
  name: file.replace(/\.vue$/, ""),
  sourceFile: `families/form/phone-field/${file}`,
  render,
  assertServerMarkup,
  assertHydratedDom,
}));
