import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick } from "vue";

import NativeSelect from "./native-select.vue";
import {
  areNativeSelectValuesEqual,
  nativeSelectSelectedValues,
  normalizeNativeSelectValue,
} from "./native-select-value.ts";
import { mountInteraction } from "../../../testing/mount.ts";

function dispatchSelectChange(select: HTMLSelectElement, value: string): void {
  select.value = value;
  select.dispatchEvent(new Event("change", { bubbles: true, cancelable: true }));
}

function dispatchMultipleSelectChange(
  select: HTMLSelectElement,
  selectedValues: readonly string[],
): void {
  const selected = new Set(selectedValues);
  for (const option of select.options) option.selected = selected.has(option.value);
  select.dispatchEvent(new Event("change", { bubbles: true, cancelable: true }));
}

const statusOptions = [
  { label: "Todo", value: "todo" },
  { label: "Doing", value: "doing" },
  { disabled: true, label: "Blocked", value: "blocked" },
] as const;

test("normalizes single and multiple values without widening order", () => {
  assert.equal(normalizeNativeSelectValue(["todo", "doing"], false), "todo");
  assert.equal(normalizeNativeSelectValue(undefined, false), "");
  assert.deepEqual(normalizeNativeSelectValue("todo", true), ["todo"]);
  assert.deepEqual(normalizeNativeSelectValue("", true), []);
  assert.deepEqual(nativeSelectSelectedValues("todo"), ["todo"]);
  assert.deepEqual(nativeSelectSelectedValues(["todo", "doing"]), ["todo", "doing"]);
  assert.equal(areNativeSelectValuesEqual(["todo", "doing"], ["todo", "doing"]), true);
  assert.equal(areNativeSelectValuesEqual(["doing", "todo"], ["todo", "doing"]), false);
});

test("renders a named native select with form and accessibility attributes", () => {
  const handle = mountInteraction(NativeSelect, {
    props: {
      id: "status",
      name: "status",
      options: statusOptions,
      ariaLabel: "Status",
      ariaDescribedby: "status-help",
      ariaErrormessage: "status-error",
      ariaInvalid: true,
      direction: "rtl",
      required: true,
    },
  });
  const select = handle.getByRole("combobox", { name: "Status" }) as HTMLSelectElement;
  const options = [...select.options];

  assert.equal(select.id, "status");
  assert.equal(select.name, "status");
  assert.equal(select.required, true);
  assert.equal(select.multiple, false);
  assert.equal(select.dir, "rtl");
  assert.equal(select.getAttribute("aria-describedby"), "status-help");
  assert.equal(select.getAttribute("aria-errormessage"), "status-error");
  assert.equal(select.getAttribute("aria-invalid"), "true");
  assert.equal(select.getAttribute("data-vize-ui"), "native-select");
  assert.equal(select.getAttribute("data-state"), "empty");
  assert.equal(select.getAttribute("data-direction"), "rtl");
  assert.equal(options[0]?.getAttribute("data-vize-ui"), "native-select-option");
  assert.equal(options[0]?.getAttribute("data-state"), "unselected");
  assert.equal(options[2]?.disabled, true);
  assert.equal(options[2]?.getAttribute("data-state"), "disabled");

  handle.exposes<{ focus: (options?: FocusOptions) => void }>().focus();
  assert.ok(handle.activeElement() === select, "exposed focus() must focus the native select");
  handle.unmount();
});

test("uncontrolled select emits model before native change", async () => {
  const handle = mountInteraction(NativeSelect, {
    props: { ariaLabel: "Status", options: statusOptions },
    record: ["update:modelValue", "change"],
  });
  const select = handle.getByRole("combobox") as HTMLSelectElement;

  dispatchSelectChange(select, "doing");
  await nextTick();

  assert.equal(select.value, "doing");
  assert.equal(select.getAttribute("data-state"), "selected");
  assert.equal(select.getAttribute("data-value"), "doing");
  assert.deepEqual(
    handle.recorded().map((emit) => [emit.event, emit.payload[0]]),
    [
      ["update:modelValue", "doing"],
      ["change", "doing"],
    ],
  );
  assert.equal(handle.recorded()[1]?.payload[1], "");
  assert.ok(handle.recorded()[1]?.payload[2] instanceof Event);
  handle.unmount();
});

test("controlled value wins until the parent accepts the request", async () => {
  const handle = mountInteraction(NativeSelect, {
    props: { ariaLabel: "Status", modelValue: "todo", options: statusOptions },
    record: ["update:modelValue", "change"],
  });
  const select = handle.getByRole("combobox") as HTMLSelectElement;

  dispatchSelectChange(select, "doing");
  await nextTick();

  assert.equal(select.value, "todo");
  assert.deepEqual(
    handle.recorded().map((emit) => [emit.event, emit.payload[0]]),
    [
      ["update:modelValue", "doing"],
      ["change", "doing"],
    ],
  );
  assert.equal(handle.recorded()[1]?.payload[1], "todo");

  await handle.wrapper.setProps({ modelValue: "doing" });
  assert.equal(select.value, "doing");
  handle.unmount();
});

test("defaultValue seeds state and native form reset restores it", async () => {
  const FormProbe = defineComponent({
    setup: () => () =>
      h("form", [
        h(NativeSelect, {
          ariaLabel: "Status",
          defaultValue: "todo",
          name: "status",
          options: statusOptions,
        }),
      ]),
  });
  const handle = mountInteraction(FormProbe);
  const form = handle.root() as HTMLFormElement;
  const select = handle.getByRole("combobox", { name: "Status" }) as HTMLSelectElement;

  assert.equal(select.value, "todo");
  assert.equal(new FormData(form).get("status"), "todo");
  dispatchSelectChange(select, "doing");
  await nextTick();
  assert.equal(select.value, "doing");
  assert.equal(new FormData(form).get("status"), "doing");

  form.reset();
  await nextTick();
  assert.equal(select.value, "todo");
  assert.equal(new FormData(form).get("status"), "todo");
  handle.unmount();
});

test("multiple mode emits selected values in native option order", async () => {
  const handle = mountInteraction(NativeSelect, {
    props: {
      ariaLabel: "Statuses",
      defaultValue: ["todo"],
      multiple: true,
      options: statusOptions,
      size: 3,
    },
    record: ["update:modelValue", "change"],
  });
  const select = handle.getByRole("listbox", { name: "Statuses" }) as HTMLSelectElement;

  assert.equal(select.multiple, true);
  assert.equal(select.getAttribute("size"), "3");
  assert.deepEqual(handle.exposes<{ selectedValues: readonly string[] }>().selectedValues, [
    "todo",
  ]);

  dispatchMultipleSelectChange(select, ["doing", "todo"]);
  await nextTick();

  assert.deepEqual(
    [...select.selectedOptions].map((option) => option.value),
    ["todo", "doing"],
  );
  assert.equal(select.getAttribute("data-selection-mode"), "multiple");
  assert.equal(select.getAttribute("data-selection-count"), "2");
  assert.equal(select.getAttribute("data-value"), null);
  assert.deepEqual(
    handle.recorded().map((emit) => [emit.event, emit.payload[0]]),
    [
      ["update:modelValue", ["todo", "doing"]],
      ["change", ["todo", "doing"]],
    ],
  );
  assert.deepEqual(handle.recorded()[1]?.payload[1], ["todo"]);
  handle.unmount();
});

test("disabled select keeps native availability semantics", async () => {
  const handle = mountInteraction(NativeSelect, {
    props: { ariaLabel: "Disabled", disabled: true, options: statusOptions },
  });
  const select = handle.getByRole("combobox") as HTMLSelectElement;

  assert.equal(select.disabled, true);
  assert.equal(select.getAttribute("data-state"), "disabled");
  assert.ok((await handle.tab()) === null);
  handle.unmount();
});

test("default slot can author native options from slot state", async () => {
  const handle = mountInteraction(NativeSelect, {
    props: { ariaLabel: "Status", defaultValue: "todo" },
    slots: {
      default: ({ selectedValues }: { selectedValues: readonly string[] }) =>
        h("option", { selected: selectedValues.includes("todo"), value: "todo" }, "Todo"),
    },
  });
  const select = handle.getByRole("combobox", { name: "Status" }) as HTMLSelectElement;

  assert.equal(select.value, "todo");
  assert.equal(select.options[0]?.textContent, "Todo");
  handle.unmount();
});

test("exposes value mutation, focus, clear, and reset controls", async () => {
  const handle = mountInteraction(NativeSelect, {
    props: { ariaLabel: "Status", defaultValue: "todo", options: statusOptions },
  });
  const select = handle.getByRole("combobox") as HTMLSelectElement;
  const exposed = handle.exposes<{
    clear: () => boolean;
    focus: (options?: FocusOptions) => void;
    reset: () => boolean;
    setValue: (value: string) => boolean;
    value: string;
  }>();

  assert.equal(exposed.setValue("doing"), true);
  await nextTick();
  assert.equal(select.value, "doing");
  assert.equal(exposed.value, "doing");

  select.blur();
  exposed.focus();
  assert.ok(handle.activeElement() === select);

  assert.equal(exposed.clear(), true);
  await nextTick();
  assert.equal(select.value, "");

  assert.equal(exposed.reset(), true);
  await nextTick();
  assert.equal(select.value, "todo");
  handle.unmount();
});
