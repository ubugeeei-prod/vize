import assert from "node:assert/strict";

import { h } from "vue";

import Fieldset from "./fieldset.vue";
import FieldsetDescription from "./fieldset-description.vue";
import FieldsetErrorMessage from "./fieldset-error-message.vue";
import FieldsetLegend from "./fieldset-legend.vue";
import type { RuntimeFixture } from "../../../conformance/runtime-conformance-fixtures.ts";

const render = () =>
  h(
    Fieldset,
    { id: "contact", hasDescription: true, invalid: true },
    {
      default: () => [
        h(FieldsetLegend, null, { default: () => "Contact" }),
        h(FieldsetDescription, null, { default: () => "How we reach you" }),
        h(FieldsetErrorMessage, null, { default: () => "Pick one" }),
      ],
    },
  );

function assertServerMarkup(html: string): void {
  assert.match(html, /^<fieldset id="contact"/);
  assert.match(html, /aria-describedby="contact-description contact-error"/);
}

function assertHydratedDom(host: HTMLElement): void {
  assert.equal(host.querySelector("legend")?.textContent, "Contact");
  assert.equal(host.querySelector('[data-vize-ui="fieldset-error-message"]')?.id, "contact-error");
}

export const fieldsetRuntimeFixtures: readonly RuntimeFixture[] = [
  "fieldset.vue",
  "fieldset-legend.vue",
  "fieldset-description.vue",
  "fieldset-error-message.vue",
].map((file) => ({
  name: file.replace(/\.vue$/, ""),
  sourceFile: `families/form/fieldset/${file}`,
  render,
  assertServerMarkup,
  assertHydratedDom,
}));
