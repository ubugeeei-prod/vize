import assert from "node:assert/strict";
import { test } from "node:test";

import { renderComposableOnServer } from "./testing/ssr-harness.ts";
import { useForm } from "./use-form.ts";
import {
  formFieldId,
  toFormFieldErrors,
  useFormErrorSummaryFields,
  useFormFieldProps,
} from "./use-form-field-props.ts";

interface Profile {
  name: string;
  address: { city: string; zip: string };
  tags: string[];
}

const initial = (): Profile => ({ name: "", address: { city: "", zip: "" }, tags: ["a"] });

async function flush(): Promise<void> {
  for (let index = 0; index < 5; index += 1) await Promise.resolve();
}

function createForm() {
  return useForm({
    initialValues: initial,
    validators: {
      name: (name) => (name.length > 0 ? undefined : "Name is required"),
      "address.city": (city) => (city.length > 0 ? undefined : ["City is required", "Pick a city"]),
    },
    validateOn: "blur",
  });
}

void test("derives path ids and ARIA relations matching the ui Field contract", () => {
  const form = createForm();
  const city = useFormFieldProps(form, "address.city", { hasDescription: true });

  assert.equal(formFieldId("tags.0"), "field-tags-0");
  assert.equal(formFieldId("", "signup"), "signup");
  assert.equal(city.id.value, "field-address-city");
  assert.deepEqual(city.labelProps.value, {
    id: "field-address-city-label",
    for: "field-address-city",
  });
  assert.deepEqual(city.descriptionProps.value, { id: "field-address-city-description" });
  assert.deepEqual(city.errorMessageProps.value, { id: "field-address-city-error" });
  assert.deepEqual(city.fieldProps.value, {
    id: "field-address-city",
    "aria-labelledby": "field-address-city-label",
    "aria-describedby": "field-address-city-description",
    "aria-errormessage": undefined,
    "aria-invalid": undefined,
  });
  const custom = useFormFieldProps(form, "name", { id: "person-name", idPrefix: "ignored" });
  assert.equal(custom.fieldProps.value.id, "person-name");
});

void test("modelProps read and write the typed path and blur marks it touched", async () => {
  const form = createForm();
  const city = useFormFieldProps(form, "address.city");

  assert.equal(city.modelProps.value.modelValue, "");
  city.modelProps.value["onUpdate:modelValue"]("Osaka");
  assert.equal(form.values.value.address.city, "Osaka");
  assert.equal(city.controlProps.value.modelValue, "Osaka");
  assert.equal(city.controlProps.value.id, "field-address-city");
  assert.equal(form.isTouched("address.city"), false);
  city.controlProps.value.onBlur();
  assert.equal(form.isTouched("address.city"), true);
  await flush();
  assert.equal(city.invalid.value, false);
});

void test("errors surface only once touched (default), after submit, or always", async () => {
  const form = createForm();
  const touched = useFormFieldProps(form, "address.city");
  const submitted = useFormFieldProps(form, "address.city", { showErrors: "submitted" });
  const always = useFormFieldProps(form, "address.city", { showErrors: "always" });
  const noErrorElement = useFormFieldProps(form, "address.city", {
    showErrors: "always",
    hasErrorMessage: false,
  });

  await form.validate();
  assert.equal(touched.invalid.value, false);
  assert.equal(submitted.invalid.value, false);
  assert.equal(always.invalid.value, true);
  assert.equal(always.errorMessage.value, "City is required");
  assert.deepEqual(always.fieldProps.value, {
    id: "field-address-city",
    "aria-labelledby": "field-address-city-label",
    "aria-describedby": "field-address-city-error",
    "aria-errormessage": "field-address-city-error",
    "aria-invalid": "true",
  });
  assert.equal(noErrorElement.fieldProps.value["aria-errormessage"], undefined);
  assert.equal(noErrorElement.fieldProps.value["aria-invalid"], "true");

  touched.modelProps.value.onBlur();
  await flush();
  assert.equal(touched.invalid.value, true);
  assert.equal(touched.errorMessage.value, "City is required");

  await form.submit();
  assert.equal(submitted.invalid.value, true);
});

void test("converts errors into ui Field errors and ordered ErrorSummary fields", async () => {
  const form = createForm();
  form.setErrors({ "tags.0": ["Bad tag"], "": ["Server rejected the form"] });
  assert.deepEqual(toFormFieldErrors(form.errors.value), [
    { name: "tags.0", message: "Bad tag", path: ["tags", 0] },
    { name: "", message: "Server rejected the form", path: [] },
  ]);

  const summary = useFormErrorSummaryFields(form, {
    order: ["name", "address.city"],
    labels: { name: "Name" },
    ids: { "address.city": "city-input" },
    rootId: "signup-form",
  });
  const eager = useFormErrorSummaryFields(form, { showErrors: "always", idPrefix: "signup" });
  assert.deepEqual(summary.value, [], "hidden until the first submit");
  assert.deepEqual(eager.value, [{ id: "signup-tags-0", message: "Bad tag" }]);

  await form.submit();
  await flush();
  assert.deepEqual(summary.value, [
    { id: "field-name", message: "Name is required", label: "Name" },
    { id: "city-input", message: "City is required" },
  ]);
  form.setErrors({ "": ["Server rejected the form"] });
  assert.deepEqual(summary.value, [{ id: "signup-form", message: "Server rejected the form" }]);
});

void test("server rendering derives the same ids and bindings without host access", async () => {
  const state = await renderComposableOnServer(() => {
    const form = createForm();
    const city = useFormFieldProps(form, "address.city");
    return { props: city.fieldProps, value: city.modelProps.value.modelValue };
  });
  assert.equal(
    state,
    '{"props":{"id":"field-address-city","aria-labelledby":"field-address-city-label"},"value":""}',
  );
});
