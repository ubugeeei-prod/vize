import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import ComboboxInput from "./combobox-input.vue";
import ComboboxRoot from "./combobox-root.vue";
import { cities, settle, type } from "./combobox-test-utils.ts";
import type { City } from "./combobox-test-utils.ts";

function mountForm(props: Record<string, unknown>) {
  const Probe = defineComponent({
    setup: () => () =>
      h("form", { id: "trip" }, [
        h(
          ComboboxRoot<City>,
          { by: "id", itemText: (city: City) => city.name, name: "city", ...props },
          () => h(ComboboxInput, { ariaLabel: "City" }),
        ),
      ]),
  });
  const handle = mountInteraction(Probe);
  const form = handle.root();
  if (!(form instanceof HTMLFormElement)) throw new Error("form root expected");
  const input = handle.getByRole("combobox", { name: "City" });
  if (!(input instanceof HTMLInputElement)) throw new Error("input expected");
  const hidden = () =>
    [...form.querySelectorAll<HTMLInputElement>("[data-vize-ui='combobox-native']")].map(
      (element) => [element.name, element.value, element.type],
    );
  return { form, handle, hidden, input };
}

test("submits the serialized selection through hidden inputs, not the visible text", async () => {
  const { form, handle, hidden, input } = mountForm({ defaultValue: cities[3] });
  await settle();
  assert.equal(input.name, "", "the visible input never submits");
  assert.deepEqual(hidden(), [["city", "4", "hidden"]]);
  assert.deepEqual([...new FormData(form).entries()], [["city", "4"]]);
  handle.unmount();
});

test("multiple selections submit one hidden input per value", async () => {
  const { handle, hidden } = mountForm({
    defaultValue: [cities[0], cities[4]],
    formValue: (city: City) => city.name,
    multiple: true,
  });
  await settle();
  assert.deepEqual(hidden(), [
    ["city", "Berlin", "hidden"],
    ["city", "Zürich", "hidden"],
  ]);
  handle.unmount();
});

test("required uses native validation on the input until something is selected", async () => {
  const { form, handle, input } = mountForm({ required: true });
  await settle();
  assert.equal(input.required, true);
  assert.equal(input.getAttribute("aria-required"), "true");
  assert.equal(form.checkValidity(), false);
  await settle();
  assert.equal(input.getAttribute("aria-invalid"), "true");
  handle.unmount();

  const filled = mountForm({ defaultValue: cities[0], required: true });
  await settle();
  assert.equal(filled.input.required, false);
  assert.equal(filled.form.checkValidity(), true);
  filled.handle.unmount();
});

test("strict required rejects unselected free text", async () => {
  const { form, handle, input } = mountForm({ required: true });
  await type(input, "unknown city");
  assert.equal(input.value, "unknown city");
  assert.equal(input.validity.customError, true);
  assert.equal(form.checkValidity(), false);
  handle.unmount();

  const free = mountForm({ required: true, strict: false });
  await type(free.input, "unknown city");
  assert.equal(free.form.checkValidity(), true);
  free.handle.unmount();
});

test("form reset restores the default selection and text", async () => {
  const { form, handle, hidden, input } = mountForm({ defaultValue: cities[1] });
  await type(input, "", "deleteContentBackward");
  assert.deepEqual(hidden(), [["city", "", "hidden"]]);
  form.reset();
  await settle();
  assert.equal(input.value, "Bogotá");
  assert.deepEqual(hidden(), [["city", "2", "hidden"]]);
  handle.unmount();
});
