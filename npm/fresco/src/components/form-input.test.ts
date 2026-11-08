import assert from "node:assert/strict";
import { test } from "vitest";
import { h, ref } from "vue";

import { dispatchKey, findNodes, mountFresco } from "../testing/mount.js";
import { Form, type FormErrors, type FormValues } from "./Form.js";

function displayValue(value: unknown): string {
  return typeof value === "string" || typeof value === "number" ? String(value) : "";
}

test("Form navigates enabled fields, validates on submit, and exposes slot state", async () => {
  const values = ref<FormValues>({ name: "", age: 15 });
  const focusedKey = ref("name");
  const invalid: FormErrors[] = [];
  const submitted: FormValues[] = [];
  const mounted = mountFresco(() =>
    h(
      Form,
      {
        fields: [
          { key: "name", label: "Name", required: true },
          { key: "internal", label: "Internal", disabled: true },
          {
            key: "age",
            label: "Age",
            validate: (value: unknown) =>
              typeof value === "number" && value >= 18 ? true : "Age must be at least 18.",
          },
        ],
        modelValue: values.value,
        focusedKey: focusedKey.value,
        focused: true,
        "onUpdate:focusedKey": (next: string) => {
          focusedKey.value = next;
        },
        onInvalid: (errors: FormErrors) => invalid.push(errors),
        onSubmit: (snapshot: FormValues) => submitted.push(snapshot),
      },
      {
        name: ({ focused, error }: { focused: boolean; error?: string }) =>
          h("input", { value: displayValue(values.value.name), focused, invalid: !!error }),
        age: ({ focused, error }: { focused: boolean; error?: string }) =>
          h("input", { value: displayValue(values.value.age), focused, invalid: !!error }),
      },
    ),
  );

  const inputs = () => findNodes(mounted.root, (node) => node.type === "input");
  assert.equal(inputs().at(0)?.props.focused, true);
  assert.equal(inputs().at(0)?.props.invalid, true);
  assert.equal(inputs().at(1)?.props.invalid, true);

  await dispatchKey({ key: "down" });
  assert.equal(focusedKey.value, "age", "disabled fields are omitted from navigation");
  assert.equal(inputs().at(1)?.props.focused, true);

  await dispatchKey({ key: "enter" });
  assert.equal(focusedKey.value, "name", "invalid submit focuses the first error");
  assert.deepEqual(invalid, [
    {
      name: "Name is required.",
      age: "Age must be at least 18.",
    },
  ]);

  values.value = { name: "Ada", age: 37 };
  await dispatchKey({ key: "enter" });
  assert.deepEqual(submitted, [{ name: "Ada", age: 37 }]);
  mounted.unmount();
});
