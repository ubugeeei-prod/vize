import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, effectScope, h, nextTick, ref } from "vue";

import type { ErrorSummaryField } from "../error-summary/error-summary.ts";
import { useErrorSummary } from "../error-summary/error-summary.ts";
import { useFieldWiring } from "../field-wiring/field-wiring.ts";
import {
  createFormErrorSummaryFields,
  formatFormFieldName,
  normalizeStandardSchemaResult,
  useFormErrorSummary,
  useFormField,
  validateStandardSchema,
  type FormFieldError,
  type StandardSchemaV1,
} from "./form.ts";
import { mountInteraction } from "../../../testing/mount.ts";

function schema<Output>(
  validate: StandardSchemaV1["~standard"]["validate"],
): StandardSchemaV1<unknown, Output> {
  return {
    "~standard": {
      validate,
      vendor: "test",
      version: 1,
    },
  };
}

const issueResult = {
  issues: [
    { message: "Enter an email", path: ["account", { key: "email" }] },
    { message: "Choose a title", path: ["items", 0, "title"] },
    { message: "Fix the form" },
  ],
} satisfies StandardSchemaV1.FailureResult;

test("formats Standard Schema paths as form field names", () => {
  const couponSymbol = Symbol("coupon");
  assert.equal(formatFormFieldName(["account", "email"]), "account.email");
  assert.equal(formatFormFieldName(["items", 0, "title"]), "items[0].title");
  assert.equal(formatFormFieldName(["first.name"]), '["first.name"]');
  assert.equal(formatFormFieldName(["fields", "first.name"]), 'fields["first.name"]');
  assert.equal(formatFormFieldName(["discounts", couponSymbol]), 'discounts["Symbol(coupon)"]');
});

test("normalizes Standard Schema failures into field and summary errors", () => {
  const result = normalizeStandardSchemaResult(issueResult, {
    idForName: (name) => (name === "" ? "checkout" : `field-${name}`),
    labelForName: (name) => (name === "account.email" ? "Email" : undefined),
    rootLabel: "Checkout",
  });

  assert.equal(result.valid, false);
  assert.deepEqual(
    result.errors.map((error) => ({
      message: error.message,
      name: error.name,
      path: error.path,
    })),
    [
      { message: "Enter an email", name: "account.email", path: ["account", "email"] },
      { message: "Choose a title", name: "items[0].title", path: ["items", 0, "title"] },
      { message: "Fix the form", name: "", path: [] },
    ],
  );
  assert.deepEqual(result.summaryFields, [
    { id: "field-account.email", label: "Email", message: "Enter an email" },
    { id: "field-items[0].title", message: "Choose a title" },
    { id: "checkout", label: "Checkout", message: "Fix the form" },
  ]);
});

test("deduplicates summary fields while preserving all field errors", () => {
  const errors: readonly FormFieldError[] = [
    { message: "Enter an email", name: "email", path: ["email"] },
    { message: "Use a company email", name: "email", path: ["email"] },
    { message: "Fix the form", name: "", path: [] },
  ];
  const fields = createFormErrorSummaryFields(errors, { rootId: "checkout" });
  assert.deepEqual(fields, [
    { id: "email", message: "Enter an email" },
    { id: "checkout", message: "Fix the form" },
  ]);
});

test("validates sync and async Standard Schemas", async () => {
  const sync = await validateStandardSchema(
    schema<{ email: string }>(() => ({ value: { email: "me@example.com" } })),
    { email: "me@example.com" },
  );
  assert.equal(sync.valid, true);
  if (sync.valid) assert.deepEqual(sync.value, { email: "me@example.com" });

  const asyncResult = await validateStandardSchema(
    schema(async () => issueResult),
    { email: "" },
    { rootId: "checkout" },
  );
  assert.equal(asyncResult.valid, false);
  assert.deepEqual(
    asyncResult.summaryFields.map((field: ErrorSummaryField) => field.id),
    ["account.email", "items[0].title", "checkout"],
  );
});

test("rejects malformed schemas, results, and options", async () => {
  await assert.rejects(() => validateStandardSchema({} as never, {}), /VIZE_UI_FORM_SCHEMA/);
  await assert.rejects(
    () =>
      validateStandardSchema(
        schema(() => ({ value: {} })),
        {},
        null as never,
      ),
    /VIZE_UI_FORM_OPTION/,
  );
  assert.throws(() => formatFormFieldName({} as never), /VIZE_UI_FORM_OPTION/);
  assert.throws(
    () => normalizeStandardSchemaResult({ issues: [{ message: 1 }] } as never),
    /VIZE_UI_FORM_SCHEMA_RESULT/,
  );
  assert.throws(
    () => normalizeStandardSchemaResult({ value: true }, null as never),
    /VIZE_UI_FORM_OPTION/,
  );
  assert.throws(
    () => createFormErrorSummaryFields([{ message: "Bad", name: "", path: [] }], { rootId: "" }),
    /VIZE_UI_FORM_OPTION/,
  );
  assert.throws(() => useFormField(null as never), /VIZE_UI_FORM_OPTION/);
  assert.throws(() => useFormErrorSummary(null as never), /VIZE_UI_FORM_OPTION/);
});

test("wires field invalid state from normalized errors", async () => {
  const errors = ref<readonly FormFieldError[]>([]);
  const probe = defineComponent({
    name: "FormFieldProbe",
    setup() {
      const field = useFormField({
        errors,
        name: "email",
      });
      const wiring = useFieldWiring({
        hasDescription: true,
        id: "email",
        invalid: field.isInvalid,
      });
      return () =>
        h("div", [
          h("label", wiring.labelProps.value, "Email"),
          h("input", { ...wiring.fieldProps.value, type: "email" }),
          h("p", wiring.descriptionProps.value, "Work email"),
          h("p", wiring.errorMessageProps.value, field.errorMessage.value),
        ]);
    },
  });
  const handle = mountInteraction(probe);
  const control = handle.root().querySelector("input");
  assert.ok(control instanceof HTMLInputElement);
  assert.equal(control.getAttribute("aria-invalid"), null);

  errors.value = [{ message: "Enter an email", name: "email", path: ["email"] }];
  await nextTick();
  assert.equal(control.getAttribute("aria-invalid"), "true");
  assert.equal(control.getAttribute("aria-errormessage"), "email-error");
  assert.equal(control.getAttribute("aria-describedby"), "email-description email-error");
  assert.equal(handle.root().querySelector("#email-error")?.textContent, "Enter an email");

  errors.value = [];
  await nextTick();
  assert.equal(control.getAttribute("aria-invalid"), null);
  assert.equal(control.getAttribute("aria-errormessage"), null);
  handle.unmount();
});

test("feeds normalized errors into an error summary controller", async () => {
  const errors = ref<readonly FormFieldError[]>([]);
  const scope = effectScope();
  const controller = scope.run(() =>
    useFormErrorSummary({
      errors,
      idForName: (name) => `field-${name}`,
      labelForName: (name) => (name === "email" ? "Email" : undefined),
    }),
  );
  assert.ok(controller);
  assert.deepEqual(controller.fields.value, []);
  assert.equal(controller.hasErrors.value, false);

  errors.value = [{ message: "Enter an email", name: "email", path: ["email"] }];
  await nextTick();
  assert.deepEqual(controller.fields.value, [
    { id: "field-email", label: "Email", message: "Enter an email" },
  ]);
  assert.equal(controller.hasErrors.value, true);

  const summary = scope.run(() =>
    useErrorSummary({
      autoFocus: false,
      fields: controller.fields,
    }),
  );
  assert.ok(summary);
  assert.equal(summary.hasErrors.value, true);
  scope.stop();
});
