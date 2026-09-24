import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { h, nextTick } from "vue";

import Fieldset from "./fieldset.vue";
import FieldsetDescription from "./fieldset-description.vue";
import FieldsetErrorMessage from "./fieldset-error-message.vue";
import FieldsetLegend from "./fieldset-legend.vue";
import type { FieldsetSlotState } from "./fieldset-types.ts";
import type { FormFieldError } from "../form/form-types.ts";
import TextInput from "../input/text-input.vue";
import { mountInteraction } from "../../../testing/mount.ts";

const addressError: FormFieldError = {
  name: "address",
  message: "Enter a full address",
  path: ["address"],
};

function mountFieldset(props: Record<string, unknown> = {}) {
  return mountInteraction(Fieldset, {
    props: { id: "address", name: "address", hasDescription: true, ...props },
    record: ["invalid-change"],
    slots: {
      default: (state: FieldsetSlotState) => [
        h(FieldsetLegend, null, { default: () => "Shipping address" }),
        h(FieldsetDescription, null, { default: () => "Where we deliver" }),
        h(TextInput, { ariaLabel: "City", name: "city" }),
        h(FieldsetErrorMessage),
        h("output", state.state),
      ],
    },
  });
}

test("renders a native fieldset named by its legend and described by its description", () => {
  const handle = mountFieldset();
  const root = handle.root();

  assert.equal(root.tagName, "FIELDSET");
  assert.equal(root.id, "address");
  assert.equal(root.getAttribute("name"), "address");
  assert.equal(root.getAttribute("data-vize-ui"), "fieldset");
  assert.equal(root.getAttribute("data-state"), "valid");
  assert.equal(root.firstElementChild?.tagName, "LEGEND");
  assert.equal(root.querySelector("legend")?.textContent, "Shipping address");
  assert.equal(
    root.querySelector('[data-vize-ui="fieldset-description"]')?.id,
    "address-description",
  );
  assert.equal(root.getAttribute("aria-describedby"), "address-description");
  assert.equal(root.querySelector('[data-vize-ui="fieldset-error-message"]'), null);
  handle.unmount();
});

test("matching form errors mark the group invalid and reference the error message", async () => {
  const handle = mountFieldset();
  const root = handle.root();

  await handle.wrapper.setProps({
    errors: [addressError, { name: "email", message: "Other", path: ["email"] }],
  });
  const error = root.querySelector('[data-vize-ui="fieldset-error-message"]');
  assert.ok(error instanceof HTMLElement);
  assert.equal(error.id, "address-error");
  assert.equal(error.textContent, "Enter a full address");
  assert.equal(root.getAttribute("data-state"), "invalid");
  assert.equal(root.getAttribute("aria-describedby"), "address-description address-error");
  await nextTick();
  assert.deepEqual(handle.recorded(), [
    { event: "invalid-change", payload: [true, [addressError]] },
  ]);

  await handle.wrapper.setProps({ hasErrorMessage: false });
  assert.equal(root.getAttribute("aria-describedby"), "address-description");
  handle.unmount();
});

test("invalid override, forceMount, and native disabled propagation", async () => {
  const handle = mountFieldset({ invalid: true, disabled: true, hasDescription: false });
  const root = handle.root();
  assert.ok(root instanceof HTMLFieldSetElement);

  assert.equal(root.disabled, true);
  assert.equal(root.getAttribute("data-state"), "disabled");
  assert.equal(root.querySelector("output")?.textContent, "disabled");
  assert.equal(root.getAttribute("aria-describedby"), "address-error");
  // Descendant controls inherit disabled from the native fieldset (browser behavior;
  // happy-dom does not implement the propagation, so only the attribute is asserted).
  assert.equal(root.hasAttribute("disabled"), true);
  handle.unmount();

  const forced = mountInteraction(Fieldset, {
    slots: {
      default: () => h(FieldsetErrorMessage, { forceMount: true }, { default: () => "Hint" }),
    },
  });
  assert.equal(
    forced.root().querySelector('[data-vize-ui="fieldset-error-message"]')?.textContent,
    "Hint",
  );
  assert.match(forced.root().id, /^vize-v-\d+-field$/);
  forced.unmount();
});

test("parts require a Fieldset provider", () => {
  const warn = console.warn;
  console.warn = () => undefined;
  try {
    for (const part of [FieldsetLegend, FieldsetDescription, FieldsetErrorMessage]) {
      assert.throws(() => mountInteraction(part), /VIZE_UI_CONTEXT_MISSING: Fieldset/);
    }
  } finally {
    console.warn = warn;
  }
});
