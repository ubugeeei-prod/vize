import assert from "node:assert/strict";

import { h, type VNode } from "vue";

import FieldRoot from "./field.vue";
import FieldDescription from "./field-description.vue";
import FieldErrorMessage from "./field-error-message.vue";
import FieldLabel from "./field-label.vue";
import type { FieldRootSlotState } from "./field-types.ts";
import type { FormFieldError } from "../form/form.ts";
import type { RuntimeFixture } from "../../../runtime-conformance-fixtures.ts";

const fieldErrors: readonly FormFieldError[] = [
  { message: "Enter an email", name: "email", path: ["email"] },
];

function renderFieldComposition(): VNode {
  return h(
    FieldRoot,
    {
      errors: fieldErrors,
      hasDescription: true,
      id: "email",
      name: "email",
    },
    {
      default: ({ fieldProps }: FieldRootSlotState) => [
        h(FieldLabel, null, { default: () => "Email" }),
        h("input", { ...fieldProps, name: "email", type: "email" }),
        h(FieldDescription, null, { default: () => "Work email" }),
        h(FieldErrorMessage),
      ],
    },
  );
}

function assertFieldServerMarkup(html: string): void {
  assert.match(html, /data-vize-ui="field"/);
  assert.match(html, /id="email"/);
  assert.match(html, /id="email-label"/);
  assert.match(html, /for="email"/);
  assert.match(html, /aria-labelledby="email-label"/);
  assert.match(html, /aria-describedby="email-description email-error"/);
  assert.match(html, /aria-errormessage="email-error"/);
  assert.match(html, /data-vize-ui="field-error-message"/);
  assert.match(html, /Enter an email/);
}

function assertFieldHydratedDom(host: HTMLElement): void {
  const field = host.querySelector('[data-vize-ui="field"]');
  const label = host.querySelector('[data-vize-ui="field-label"]');
  const input = host.querySelector("input");
  const description = host.querySelector('[data-vize-ui="field-description"]');
  const error = host.querySelector('[data-vize-ui="field-error-message"]');
  assert.ok(field instanceof HTMLElement);
  assert.ok(label instanceof HTMLLabelElement);
  assert.ok(input instanceof HTMLInputElement);
  assert.ok(description instanceof HTMLElement);
  assert.ok(error instanceof HTMLElement);
  assert.equal(field.getAttribute("data-state"), "invalid");
  assert.equal(input.getAttribute("aria-labelledby"), label.id);
  assert.equal(input.getAttribute("aria-describedby"), `${description.id} ${error.id}`);
  assert.equal(input.getAttribute("aria-errormessage"), error.id);
  assert.equal(error.textContent, "Enter an email");
}

const fieldRuntimeFixture = {
  name: "field",
  sourceFile: "families/form/field/field.vue",
  render: renderFieldComposition,
  assertServerMarkup: assertFieldServerMarkup,
  assertHydratedDom: assertFieldHydratedDom,
} satisfies RuntimeFixture;

const fieldDescriptionRuntimeFixture = {
  name: "field-description",
  sourceFile: "families/form/field/field-description.vue",
  render: renderFieldComposition,
  assertServerMarkup: assertFieldServerMarkup,
  assertHydratedDom: assertFieldHydratedDom,
} satisfies RuntimeFixture;

const fieldErrorMessageRuntimeFixture = {
  name: "field-error-message",
  sourceFile: "families/form/field/field-error-message.vue",
  render: renderFieldComposition,
  assertServerMarkup: assertFieldServerMarkup,
  assertHydratedDom: assertFieldHydratedDom,
} satisfies RuntimeFixture;

const fieldLabelRuntimeFixture = {
  name: "field-label",
  sourceFile: "families/form/field/field-label.vue",
  render: renderFieldComposition,
  assertServerMarkup: assertFieldServerMarkup,
  assertHydratedDom: assertFieldHydratedDom,
} satisfies RuntimeFixture;

export const fieldRuntimeFixtures: readonly RuntimeFixture[] = [
  fieldRuntimeFixture,
  fieldDescriptionRuntimeFixture,
  fieldErrorMessageRuntimeFixture,
  fieldLabelRuntimeFixture,
];
