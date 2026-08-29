import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import FieldRoot from "./field.vue";
import FieldDescription from "./field-description.vue";
import FieldErrorMessage from "./field-error-message.vue";
import FieldLabel from "./field-label.vue";
import type { FieldRootExpose, FieldRootSlotState } from "./field-types.ts";
import type { FormFieldError } from "./form.ts";
import TextInput from "./text-input.vue";
import { mountInteraction } from "./testing/mount.ts";

const emailError: FormFieldError = {
  message: "Enter an email",
  name: "email",
  path: ["email"],
};

function queryField(root: HTMLElement): {
  readonly control: HTMLInputElement;
  readonly description: HTMLElement | null;
  readonly error: HTMLElement | null;
  readonly label: HTMLLabelElement;
} {
  const control = root.querySelector('[data-vize-ui="input"]');
  const label = root.querySelector('[data-vize-ui="field-label"]');
  const description = root.querySelector('[data-vize-ui="field-description"]');
  const error = root.querySelector('[data-vize-ui="field-error-message"]');
  assert.ok(control instanceof HTMLInputElement);
  assert.ok(label instanceof HTMLLabelElement);
  assert.ok(description === null || description instanceof HTMLElement);
  assert.ok(error === null || error instanceof HTMLElement);
  return { control, description, error, label };
}

test("wires label and described-by props through public SFCs", () => {
  const handle = mountInteraction(FieldRoot, {
    props: { hasDescription: true, id: "email", name: "email" },
    slots: {
      default: ({ fieldProps }: FieldRootSlotState) => [
        h(FieldLabel, null, { default: () => "Email" }),
        h(TextInput, { ...fieldProps, name: "email", type: "email" }),
        h(FieldDescription, null, { default: () => "Work email" }),
        h(FieldErrorMessage),
      ],
    },
  });
  const { control, description, error, label } = queryField(handle.root());

  assert.equal(handle.root().getAttribute("data-vize-ui"), "field");
  assert.equal(handle.root().getAttribute("part"), "root");
  assert.equal(handle.root().getAttribute("data-state"), "valid");
  assert.equal(handle.root().getAttribute("data-invalid"), "false");
  assert.equal(control.id, "email");
  assert.equal(label.id, "email-label");
  assert.equal(label.htmlFor, "email");
  assert.equal(control.getAttribute("aria-labelledby"), "email-label");
  assert.equal(description?.id, "email-description");
  assert.equal(control.getAttribute("aria-describedby"), "email-description");
  assert.equal(control.getAttribute("aria-invalid"), null);
  assert.equal(control.getAttribute("aria-errormessage"), null);
  assert.equal(error, null);
  assert.equal(handle.root().querySelector('[data-vize-ui="field-error-message-host"]'), null);

  const exposed = handle.exposes<FieldRootExpose>();
  assert.equal(exposed.id, "email");
  assert.equal(exposed.name, "email");
  assert.equal(exposed.invalid, false);
  assert.equal(exposed.fieldProps.id, "email");
  handle.unmount();
});

test("renders normalized form errors and emits invalid changes", async () => {
  const handle = mountInteraction(FieldRoot, {
    props: {
      errors: [],
      hasDescription: true,
      id: "email",
      name: "email",
    },
    record: ["invalid-change"],
    slots: {
      default: ({ fieldProps }: FieldRootSlotState) => [
        h(FieldLabel, null, { default: () => "Email" }),
        h(TextInput, { ...fieldProps, name: "email", type: "email" }),
        h(FieldDescription, null, { default: () => "Work email" }),
        h(FieldErrorMessage),
      ],
    },
  });
  const { control } = queryField(handle.root());

  await handle.wrapper.setProps({ errors: [emailError] });
  await nextTick();

  const { error } = queryField(handle.root());
  assert.equal(handle.root().getAttribute("data-state"), "invalid");
  assert.equal(control.getAttribute("aria-invalid"), "true");
  assert.equal(control.getAttribute("aria-errormessage"), "email-error");
  assert.equal(control.getAttribute("aria-describedby"), "email-description email-error");
  assert.equal(error?.id, "email-error");
  assert.equal(error?.textContent, "Enter an email");
  assert.deepEqual(handle.recorded(), [{ event: "invalid-change", payload: [true, [emailError]] }]);

  await handle.wrapper.setProps({ errors: [] });
  await nextTick();
  assert.equal(control.getAttribute("aria-invalid"), null);
  assert.equal(handle.root().querySelector('[data-vize-ui="field-error-message"]'), null);
  assert.equal(handle.root().querySelector('[data-vize-ui="field-error-message-host"]'), null);
  assert.equal(handle.recorded()[1]?.event, "invalid-change");
  assert.deepEqual(handle.recorded()[1]?.payload, [false, []]);
  handle.unmount();
});

test("keeps forced error messages mounted without invalid ARIA", () => {
  const handle = mountInteraction(FieldRoot, {
    props: { hasErrorMessage: true, id: "email", name: "email" },
    slots: {
      default: ({ fieldProps }: FieldRootSlotState) => [
        h(FieldLabel, null, { default: () => "Email" }),
        h(TextInput, { ...fieldProps, name: "email", type: "email" }),
        h(FieldErrorMessage, { forceMount: true }, { default: () => "No errors" }),
      ],
    },
  });
  const { control, error } = queryField(handle.root());

  assert.equal(control.getAttribute("aria-invalid"), null);
  assert.equal(control.getAttribute("aria-errormessage"), null);
  assert.equal(control.getAttribute("aria-describedby"), null);
  assert.equal(error?.id, "email-error");
  assert.equal(error?.getAttribute("data-state"), "valid");
  assert.equal(error?.textContent, "No errors");
  handle.unmount();
});

test("allows direct invalid overrides for native validation", () => {
  const handle = mountInteraction(FieldRoot, {
    props: { hasErrorMessage: true, id: "email", invalid: true, name: "email" },
    slots: {
      default: ({ fieldProps }: FieldRootSlotState) => [
        h(FieldLabel, null, { default: () => "Email" }),
        h(TextInput, { ...fieldProps, name: "email", type: "email" }),
        h(FieldErrorMessage, null, {
          default: ({ invalid }: { readonly invalid: boolean }) =>
            invalid ? "Use a work email" : "",
        }),
      ],
    },
  });
  const { control, error } = queryField(handle.root());

  assert.equal(control.getAttribute("aria-invalid"), "true");
  assert.equal(control.getAttribute("aria-errormessage"), "email-error");
  assert.equal(error?.textContent, "Use a work email");
  handle.unmount();
});

test("suppresses optional ARIA relations when declared absent", () => {
  const handle = mountInteraction(FieldRoot, {
    props: {
      hasDescription: false,
      hasErrorMessage: false,
      id: "email",
      invalid: true,
      name: "email",
    },
    slots: {
      default: ({ fieldProps }: FieldRootSlotState) => [
        h(FieldLabel, null, { default: () => "Email" }),
        h(TextInput, { ...fieldProps, name: "email", type: "email" }),
        h(FieldDescription, null, { default: () => "Work email" }),
        h(FieldErrorMessage, null, { default: () => "Invalid" }),
      ],
    },
  });
  const { control, description, error } = queryField(handle.root());

  assert.equal(description?.id, "email-description");
  assert.equal(error?.id, "email-error");
  assert.equal(control.getAttribute("aria-invalid"), "true");
  assert.equal(control.getAttribute("aria-describedby"), null);
  assert.equal(control.getAttribute("aria-errormessage"), null);
  handle.unmount();
});

test("rejects field parts outside a Field root", () => {
  assert.throws(() => mountInteraction(FieldLabel), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(FieldDescription), /VIZE_UI_CONTEXT_MISSING/);
  assert.throws(() => mountInteraction(FieldErrorMessage), /VIZE_UI_CONTEXT_MISSING/);
});
