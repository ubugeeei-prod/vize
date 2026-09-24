import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import SelectRoot from "./select-root.vue";
import SelectTrigger from "./select-trigger.vue";
import SelectValue from "./select-value.vue";
import { fruits, settle } from "./select-test-utils.ts";
import type { Fruit } from "./select-test-utils.ts";

function mountForm(selectProps: Record<string, unknown>, outside = false) {
  const Probe = defineComponent({
    setup: () => () => [
      h("form", { id: "checkout" }, [
        outside
          ? null
          : h(
              SelectRoot<Fruit>,
              { by: "id", itemText: (fruit: Fruit) => fruit.name, name: "fruit", ...selectProps },
              () => [h(SelectTrigger, { ariaLabel: "Fruit" }, () => h(SelectValue))],
            ),
      ]),
      outside
        ? h(
            SelectRoot<Fruit>,
            { by: "id", form: "checkout", name: "fruit", ...selectProps },
            () => [h(SelectTrigger, { ariaLabel: "Fruit" }, () => h(SelectValue))],
          )
        : null,
    ],
  });
  const handle = mountInteraction(
    defineComponent({ setup: () => () => h("div", { id: "host" }, [h(Probe)]) }),
  );
  const form = document.querySelector<HTMLFormElement>("form#checkout");
  if (form === null) throw new Error("form missing");
  const native = document.querySelector<HTMLSelectElement>("[data-vize-ui='select-native']");
  if (native === null) throw new Error("native select missing");
  return { form, handle, native };
}

test("submits the serialized `by` key through a hidden native select", async () => {
  const { form, handle, native } = mountForm({ defaultValue: fruits[1] });
  await settle();
  assert.equal(native.hidden, true);
  assert.equal(native.getAttribute("aria-hidden"), "true");
  assert.equal(native.tabIndex, -1);
  assert.deepEqual([...new FormData(form).entries()], [["fruit", "2"]]);
  handle.unmount();
});

test("empty single selections submit an empty value and fail required validation", async () => {
  const { form, handle, native } = mountForm({ required: true });
  await settle();
  assert.deepEqual([...new FormData(form).entries()], [["fruit", ""]]);
  assert.equal(native.required, true);
  assert.equal(form.checkValidity(), false);
  await nextTick();
  const trigger = handle.getByRole("combobox", { name: "Fruit" });
  assert.equal(document.activeElement, trigger, "invalid focuses the visible trigger");
  assert.equal(trigger.getAttribute("aria-invalid"), "true");
  assert.equal(
    handle.root().querySelector("[data-vize-ui='select']")?.getAttribute("data-invalid"),
    "true",
  );
  handle.unmount();
});

test("multiple selections submit one entry per value with a custom serializer", async () => {
  const { handle, native } = mountForm({
    defaultValue: [fruits[0], fruits[4]],
    formValue: (fruit: Fruit) => fruit.name.toLowerCase(),
    items: fruits,
    multiple: true,
  });
  await settle();
  assert.equal(native.multiple, true);
  // happy-dom's FormData serializes only the first selected option of a
  // multiple select, so the submitted set is asserted through the control.
  assert.deepEqual(
    [...native.selectedOptions].map((option) => option.value),
    ["apple", "date"],
  );
  assert.deepEqual(
    [...native.options].map((option) => option.value),
    ["apple", "banana", "blueberry", "cherry", "date"],
  );
  handle.unmount();
});

test("disabled selects are excluded from submission", async () => {
  const { handle, native } = mountForm({ defaultValue: fruits[0], disabled: true });
  await settle();
  // Browsers skip disabled controls when building the form data set; happy-dom
  // does not, so the contract is asserted on the control itself.
  assert.equal(native.disabled, true);
  assert.equal(native.name, "fruit");
  handle.unmount();
});

test("the form attribute associates a select rendered outside its form", async () => {
  const { form, handle, native } = mountForm({ defaultValue: fruits[2] }, true);
  await settle();
  assert.equal(native.form, form);
  assert.deepEqual([...new FormData(form).entries()], [["fruit", "3"]]);
  handle.unmount();
});

test("form reset restores the default selection", async () => {
  const { form, handle, native } = mountForm({ defaultValue: fruits[0], items: fruits });
  await settle();
  native.value = "5";
  native.dispatchEvent(new Event("change", { bubbles: true }));
  await settle();
  assert.deepEqual([...new FormData(form).entries()], [["fruit", "5"]]);
  assert.equal(handle.getByRole("combobox", { name: "Fruit" }).textContent, "Date");

  form.reset();
  await settle();
  assert.equal(handle.getByRole("combobox", { name: "Fruit" }).textContent, "Apple");
  assert.deepEqual([...new FormData(form).entries()], [["fruit", "1"]]);
  handle.unmount();
});
