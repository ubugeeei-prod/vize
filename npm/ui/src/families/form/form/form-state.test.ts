import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { effectScope, nextTick } from "vue";

import {
  normalizeNativeConstraintErrors,
  useFormState,
  type FormFieldError,
  type StandardSchemaV1,
} from "./form.ts";

function schema<Input, Output>(
  validate: StandardSchemaV1<Input, Output>["~standard"]["validate"],
): StandardSchemaV1<Input, Output> {
  return {
    "~standard": {
      validate,
      vendor: "test",
      version: 1,
    },
  };
}

test("normalizes native constraint validation failures", () => {
  const form = document.createElement("form");
  const unnamed = document.createElement("input");
  const email = document.createElement("input");
  email.name = "account.email";
  email.setCustomValidity("Use a work email");
  unnamed.setCustomValidity("Ignored");
  form.append(unnamed, email);
  document.body.append(form);

  const errors = normalizeNativeConstraintErrors(form);
  assert.deepEqual(errors, [
    { message: "Use a work email", name: "account.email", path: ["account.email"] },
  ]);

  const renamed = normalizeNativeConstraintErrors(form, {
    messageForControl: () => "Server-compatible email required",
    nameForControl: (control) => {
      const name = control.getAttribute("name");
      return name === null ? null : `native:${name}`;
    },
  });
  assert.deepEqual(renamed, [
    {
      message: "Server-compatible email required",
      name: "native:account.email",
      path: ["native:account.email"],
    },
  ]);
  form.remove();
});

test("tracks field state and focuses the first registered invalid field", async () => {
  const controller = useFormState();
  const email = document.createElement("input");
  const password = document.createElement("input");
  email.name = "email";
  password.name = "password";
  document.body.append(email, password);
  const unregisterEmail = controller.registerField("email", email);
  controller.registerField("password", password);

  const emailState = controller.fieldState("email");
  assert.equal(emailState.value.isDirty, false);
  assert.equal(controller.isDirty.value, false);

  controller.markFieldVisited("email");
  controller.markFieldDirty("email");
  controller.markFieldTouched("email");
  controller.setServerErrors([
    { message: "Enter a password", name: "password", path: ["password"] },
    { message: "Enter an email", name: "email", path: ["email"] },
  ]);
  await nextTick();

  assert.equal(emailState.value.isDirty, true);
  assert.equal(emailState.value.isTouched, true);
  assert.equal(emailState.value.isVisited, true);
  assert.equal(emailState.value.isInvalid, true);
  assert.equal(emailState.value.errorMessage, "Enter an email");
  assert.equal(controller.isDirty.value, true);
  assert.equal(controller.isTouched.value, true);
  assert.equal(controller.isVisited.value, true);
  assert.equal(controller.focusFirstInvalid(), true);
  assert.equal(document.activeElement, password);

  unregisterEmail();
  controller.reset();
  assert.equal(controller.hasErrors.value, false);
  assert.equal(controller.isDirty.value, false);
  email.remove();
  password.remove();
});

test("discards stale async validation results", async () => {
  type Input = { readonly email: string };
  type Output = Input & { readonly normalized: true };
  const resolvers: Array<(result: StandardSchemaV1.Result<Output>) => void> = [];
  const controller = useFormState({
    schema: schema<Input, Output>(
      () =>
        new Promise((resolve) => {
          resolvers.push(resolve);
        }),
    ),
  });

  const slow = controller.validate({ email: "slow@example.com" });
  const fast = controller.validate({ email: "fast@example.com" });
  assert.equal(controller.isValidating.value, true);
  resolvers[1]?.({ value: { email: "fast@example.com", normalized: true } });
  const fastResult = await fast;
  assert.equal(fastResult.valid, true);
  assert.equal(controller.hasErrors.value, false);
  assert.equal(controller.isValidating.value, true);

  resolvers[0]?.({
    issues: [{ message: "Too slow", path: ["email"] }],
  });
  const slowResult = await slow;
  assert.equal(slowResult.valid, false);
  assert.deepEqual(controller.errors.value, []);
  assert.equal(controller.isValidating.value, false);
});

test("copies stored error paths before exposing field state", () => {
  const path = ["email"];
  const controller = useFormState({
    initialErrors: [{ message: "Bad email", name: "email", path }],
  });
  const emailState = controller.fieldState("email");

  path.push("mutated");
  assert.deepEqual(controller.errors.value, [
    { message: "Bad email", name: "email", path: ["email"] },
  ]);
  assert.deepEqual(emailState.value.errors, [
    { message: "Bad email", name: "email", path: ["email"] },
  ]);
  assert.equal(Object.isFrozen(controller.errors.value[0]?.path), true);
});

test("reset invalidates in-flight submit results", async () => {
  type Input = { readonly email: string };
  type Output = Input & { readonly normalized: true };
  const submitted: Output[] = [];
  let resolveSubmit: ((result: StandardSchemaV1.Result<Output>) => void) | undefined;
  const controller = useFormState<Input, Output>({
    onSubmit: (value) => submitted.push(value),
    schema: schema(
      () =>
        new Promise((resolve) => {
          resolveSubmit = resolve;
        }),
    ),
  });

  const pendingSubmit = controller.submit({ email: "me@example.com" });
  controller.reset();
  resolveSubmit?.({ value: { email: "me@example.com", normalized: true } });

  const result = await pendingSubmit;
  assert.equal(result.valid, true);
  assert.deepEqual(submitted, []);
  assert.deepEqual(controller.errors.value, []);
  assert.equal(controller.isSubmitting.value, false);
});

test("submits only the latest valid result", async () => {
  type Input = { readonly email: string };
  type Output = Input & { readonly normalized: true };
  const submitted: Output[] = [];
  const email = document.createElement("input");
  email.name = "email";
  document.body.append(email);
  const controller = useFormState<Input, Output>({
    onSubmit: (value) => submitted.push(value),
    schema: schema((value) => {
      const input = value as Input;
      return input.email.includes("@")
        ? { value: { email: input.email.toLowerCase(), normalized: true } }
        : { issues: [{ message: "Enter an email", path: ["email"] }] };
    }),
  });
  controller.registerField("email", email);

  const invalid = await controller.submit({ email: "bad" });
  assert.equal(invalid.valid, false);
  assert.equal(document.activeElement, email);
  assert.deepEqual(submitted, []);
  assert.equal(controller.submitCount.value, 1);

  const valid = await controller.submit({ email: "ME@EXAMPLE.COM" });
  assert.equal(valid.valid, true);
  assert.deepEqual(submitted, [{ email: "me@example.com", normalized: true }]);
  assert.equal(controller.isSubmitting.value, false);
  email.remove();
});

test("rejects malformed state and native constraint inputs", async () => {
  assert.throws(() => useFormState(null as never), /VIZE_UI_FORM_STATE/);
  assert.throws(
    () => normalizeNativeConstraintErrors(document.createElement("div") as never),
    /VIZE_UI_FORM_CONSTRAINT/,
  );

  const controller = useFormState({
    initialErrors: [{ message: "Bad", name: "email", path: ["email"] }],
  });
  assert.throws(
    () => controller.registerField("", document.createElement("input")),
    /VIZE_UI_FORM_STATE/,
  );
  assert.throws(
    () => controller.setErrors([{ message: "Bad", name: "email", path: "email" }] as never),
    /VIZE_UI_FORM_STATE/,
  );
  await assert.rejects(() => controller.validate("value"), /VIZE_UI_FORM_STATE/);
});

test("keeps server errors in the shared summary pipeline", () => {
  const serverErrors: readonly FormFieldError[] = [
    { message: "Email already exists", name: "email", path: ["email"] },
    { message: "Try another email", name: "email", path: ["email"] },
  ];
  const scope = effectScope();
  const controller = scope.run(() =>
    useFormState({
      initialErrors: serverErrors,
      labelForName: (name) => (name === "email" ? "Email" : undefined),
    }),
  );
  assert.ok(controller);
  assert.deepEqual(controller.summaryFields.value, [
    { id: "email", label: "Email", message: "Email already exists" },
  ]);
  controller.clearErrors();
  assert.deepEqual(controller.summaryFields.value, []);
  scope.stop();
});
