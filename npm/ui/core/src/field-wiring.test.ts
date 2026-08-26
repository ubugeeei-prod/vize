import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref, type Ref } from "vue";

import { useFieldWiring } from "./field-wiring.ts";
import { mountInteraction } from "./testing/mount.ts";

interface ProbeState {
  readonly invalid: Ref<boolean>;
  readonly hasDescription: Ref<boolean>;
  readonly hasErrorMessage: Ref<boolean>;
}

function createProbe(id?: string): { component: ReturnType<typeof defineComponent> } & ProbeState {
  const invalid = ref(false);
  const hasDescription = ref(false);
  const hasErrorMessage = ref(true);
  const component = defineComponent({
    name: "FieldWiringProbe",
    setup() {
      const wiring = useFieldWiring({ hasDescription, hasErrorMessage, id, invalid });
      return () =>
        h("div", [
          h("label", wiring.labelProps.value, "Email"),
          h("input", { ...wiring.fieldProps.value, type: "email" }),
          h("p", wiring.descriptionProps.value, "We never share it."),
          h("p", wiring.errorMessageProps.value, "Enter a valid address."),
        ]);
    },
  });
  return { component, hasDescription, hasErrorMessage, invalid };
}

function queryControl(root: HTMLElement): {
  control: HTMLInputElement;
  label: HTMLLabelElement;
  description: HTMLElement;
  error: HTMLElement;
} {
  const control = root.querySelector("input");
  const label = root.querySelector("label");
  const [description, error] = root.querySelectorAll("p");
  assert.ok(control instanceof HTMLInputElement);
  assert.ok(label instanceof HTMLLabelElement);
  assert.ok(description instanceof HTMLElement);
  assert.ok(error instanceof HTMLElement);
  return { control, description, error, label };
}

test("wires the label to the control", () => {
  const probe = createProbe();
  const handle = mountInteraction(probe.component);
  const { control, label } = queryControl(handle.root());
  assert.ok(control.id.length > 0);
  assert.equal(label.getAttribute("for"), control.id);
  assert.equal(control.getAttribute("aria-labelledby"), label.id);
  assert.equal(label.id, `${control.id}-label`);
  assert.equal(control.getAttribute("aria-describedby"), null);
  assert.equal(control.getAttribute("aria-invalid"), null);
  assert.equal(control.getAttribute("aria-errormessage"), null);
  handle.unmount();
});

test("describes the control while a description exists", async () => {
  const probe = createProbe();
  const handle = mountInteraction(probe.component);
  probe.hasDescription.value = true;
  await nextTick();
  const { control, description } = queryControl(handle.root());
  assert.equal(control.getAttribute("aria-describedby"), description.id);
  assert.equal(description.id, `${control.id}-description`);
  handle.unmount();
});

test("wires the error message while invalid", async () => {
  const probe = createProbe();
  probe.hasDescription.value = true;
  const handle = mountInteraction(probe.component);
  probe.invalid.value = true;
  await nextTick();
  const { control, description, error } = queryControl(handle.root());
  assert.equal(control.getAttribute("aria-invalid"), "true");
  assert.equal(control.getAttribute("aria-errormessage"), error.id);
  assert.equal(control.getAttribute("aria-describedby"), `${description.id} ${error.id}`);
  assert.equal(error.id, `${control.id}-error`);

  probe.invalid.value = false;
  await nextTick();
  assert.equal(control.getAttribute("aria-invalid"), null);
  assert.equal(control.getAttribute("aria-errormessage"), null);
  assert.equal(control.getAttribute("aria-describedby"), description.id);
  handle.unmount();
});

test("omits error wiring without an error element", async () => {
  const probe = createProbe();
  probe.hasErrorMessage.value = false;
  const handle = mountInteraction(probe.component);
  probe.invalid.value = true;
  await nextTick();
  const { control } = queryControl(handle.root());
  assert.equal(control.getAttribute("aria-invalid"), "true");
  assert.equal(control.getAttribute("aria-errormessage"), null);
  assert.equal(control.getAttribute("aria-describedby"), null);
  handle.unmount();
});

test("derives every id from a consumer-owned id", () => {
  const probe = createProbe("billing-email");
  const handle = mountInteraction(probe.component);
  const { control, description, error, label } = queryControl(handle.root());
  assert.equal(control.id, "billing-email");
  assert.equal(label.id, "billing-email-label");
  assert.equal(description.id, "billing-email-description");
  assert.equal(error.id, "billing-email-error");
  handle.unmount();
});

test("rejects use outside component setup", () => {
  assert.throws(() => useFieldWiring(), /VIZE_UI_FIELD_WIRING_SETUP/);
});

test("rejects invalid options", () => {
  const probe = defineComponent({
    name: "FieldWiringInvalidOptionProbe",
    setup() {
      const wiring = useFieldWiring({ invalid: "yes" as never });
      return () => h("input", wiring.fieldProps.value);
    },
  });
  assert.throws(() => mountInteraction(probe), /VIZE_UI_FIELD_WIRING_OPTION/);
});
