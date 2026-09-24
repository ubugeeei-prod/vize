import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";

import CheckboxGroup from "./checkbox-group.vue";
import CheckboxGroupItem from "./checkbox-group-item.vue";
import CheckboxGroupSelectAll from "./checkbox-group-select-all.vue";
import type { CheckboxGroupExpose, CheckboxGroupSlotState } from "./checkbox-group-types.ts";
import { mountInteraction } from "../../../testing/mount.ts";

interface Topping {
  readonly id: number;
  readonly label: string;
}

const toppings: readonly Topping[] = [
  { id: 1, label: "Cheese" },
  { id: 2, label: "Olives" },
  { id: 3, label: "Basil" },
];

function renderGroup(options: readonly unknown[]) {
  return (state: CheckboxGroupSlotState<unknown>) => [
    h("label", [h(CheckboxGroupSelectAll), "All"]),
    ...options.map((option, index) =>
      h("label", { key: index }, [h(CheckboxGroupItem, { value: option }), `Option ${index}`]),
    ),
    h("output", `${state.state}:${state.values.length}`),
  ];
}

function items(root: Element): HTMLInputElement[] {
  return [...root.querySelectorAll<HTMLInputElement>('[data-vize-ui="checkbox-group-item"]')];
}

function selectAll(root: Element): HTMLInputElement {
  const input = root.querySelector('[data-vize-ui="checkbox-group-select-all"]');
  assert.ok(input instanceof HTMLInputElement);
  return input;
}

async function toggle(input: HTMLInputElement): Promise<void> {
  input.click();
  await nextTick();
  await Promise.resolve();
}

test("renders a labelled group of named native checkboxes with typed values", () => {
  const handle = mountInteraction(CheckboxGroup, {
    props: {
      options: ["red", "green", "blue"],
      defaultValue: ["green"],
      name: "colors",
      ariaLabel: "Colors",
      ariaDescribedby: "colors-help",
    },
    slots: { default: renderGroup(["red", "green", "blue"]) },
  });
  const root = handle.root();
  const [red, green, blue] = items(root);
  assert.ok(red && green && blue);

  assert.equal(root.getAttribute("role"), "group");
  assert.equal(root.getAttribute("aria-label"), "Colors");
  assert.equal(root.getAttribute("data-vize-ui"), "checkbox-group");
  assert.equal(root.getAttribute("data-state"), "some");
  assert.equal(red.name, "colors");
  assert.equal(red.value, "red");
  assert.equal(green.checked, true);
  assert.equal(green.getAttribute("data-state"), "checked");
  assert.equal(blue.getAttribute("aria-describedby"), "colors-help");
  assert.equal(root.querySelector("output")?.textContent, "some:1");
  handle.unmount();
});

test("toggling items updates values in option order and emits changes", async () => {
  const handle = mountInteraction(CheckboxGroup, {
    props: { options: toppings, ariaLabel: "Toppings" },
    record: ["update:modelValue", "change"],
    slots: { default: renderGroup(toppings) },
  });
  const [cheese, olives, basil] = items(handle.root());
  assert.ok(cheese && olives && basil);

  await toggle(basil);
  await toggle(cheese);
  assert.deepEqual(handle.exposes<CheckboxGroupExpose<Topping>>().values, [
    toppings[0],
    toppings[2],
  ]);
  await toggle(basil);
  assert.deepEqual(handle.recorded().at(-1), {
    event: "change",
    payload: [[toppings[0]], toppings[2], false],
  });
  assert.equal(cheese.value, "0", "objects submit their option index by default");
  handle.unmount();
});

test("select-all is tri-state and toggles every enabled option", async () => {
  const handle = mountInteraction(CheckboxGroup, {
    props: {
      options: ["a", "b", "c"],
      defaultValue: ["c"],
      isOptionDisabled: (value: string) => value === "c",
      ariaLabel: "Letters",
    },
    record: ["change"],
    slots: { default: renderGroup(["a", "b", "c"]) },
  });
  const root = handle.root();
  const parent = selectAll(root);
  const [a, b, c] = items(root);
  assert.ok(a && b && c);

  assert.equal(c.disabled, true);
  assert.equal(parent.getAttribute("aria-checked"), "false", "disabled selections do not count");
  await toggle(a);
  assert.equal(parent.indeterminate, true);
  assert.equal(parent.getAttribute("aria-checked"), "mixed");
  assert.equal(parent.getAttribute("data-state"), "indeterminate");
  await toggle(parent);
  assert.equal(parent.checked, true);
  assert.equal(parent.getAttribute("aria-checked"), "true");
  assert.deepEqual(handle.exposes<CheckboxGroupExpose<string>>().values, ["a", "b", "c"]);
  await toggle(parent);
  assert.deepEqual(
    handle.exposes<CheckboxGroupExpose<string>>().values,
    ["c"],
    "disabled options keep their selection",
  );
  assert.equal(parent.checked, false);
  assert.deepEqual(handle.recorded().at(-1), { event: "change", payload: [["c"], null, false] });
  handle.unmount();
});

test("controlled values win and `by` matches fresh object copies", async () => {
  const handle = mountInteraction(CheckboxGroup, {
    props: {
      options: toppings,
      modelValue: [{ id: 2, label: "Olives" }],
      by: (topping: Topping) => topping.id,
      getFormValue: (topping: Topping) => String(topping.id),
      name: "toppings",
      ariaLabel: "Toppings",
    },
    record: ["update:modelValue"],
    slots: { default: renderGroup(toppings) },
  });
  const [cheese, olives] = items(handle.root());
  assert.ok(cheese && olives);

  assert.equal(olives.checked, true);
  assert.equal(olives.value, "2");
  await toggle(cheese);
  assert.deepEqual(handle.recorded()[0], {
    event: "update:modelValue",
    payload: [[toppings[0], toppings[1]]],
  });
  assert.equal(cheese.checked, false, "controlled value wins until accepted");
  await handle.wrapper.setProps({ modelValue: [toppings[0], toppings[1]] });
  assert.equal(cheese.checked, true);
  handle.unmount();
});

test("submits selected values, requires one selection, and restores defaults on reset", async () => {
  const Probe = defineComponent({
    setup: () => () =>
      h("form", [
        h(
          CheckboxGroup,
          {
            options: ["x", "y"],
            defaultValue: ["x"],
            name: "pick",
            required: true,
            ariaLabel: "Pick",
          },
          { default: renderGroup(["x", "y"]) },
        ),
      ]),
  });
  const handle = mountInteraction(Probe);
  const form = handle.root();
  assert.ok(form instanceof HTMLFormElement);
  const [x, y] = items(form);
  assert.ok(x && y);

  assert.deepEqual(new FormData(form).getAll("pick"), ["x"]);
  assert.equal(x.required, false, "required clears once something is selected");
  await toggle(x);
  assert.equal(x.required, true);
  assert.equal(y.required, true);
  await toggle(y);
  assert.deepEqual(new FormData(form).getAll("pick"), ["y"]);
  form.reset();
  await nextTick();
  await nextTick();
  assert.deepEqual(new FormData(form).getAll("pick"), ["x"]);
  assert.equal(x.checked, true);
  assert.equal(y.checked, false);
  handle.unmount();
});

test("disabled groups disable every checkbox and ignore toggles", async () => {
  const handle = mountInteraction(CheckboxGroup, {
    props: { options: ["a", "b"], disabled: true, ariaLabel: "Letters" },
    record: ["update:modelValue"],
    slots: { default: renderGroup(["a", "b"]) },
  });
  const root = handle.root();
  assert.equal(root.getAttribute("data-state"), "disabled");
  assert.equal(root.getAttribute("aria-disabled"), "true");
  assert.ok(items(root).every((input) => input.disabled));
  assert.equal(selectAll(root).disabled, true);
  assert.equal(handle.exposes<CheckboxGroupExpose<string>>().setAll(true), false);
  assert.deepEqual(handle.recorded(), []);
  handle.unmount();
});

test("exposes isSelected, setSelected, setAll, reset, and the selection summary", () => {
  const selected = ref<readonly string[]>([]);
  const handle = mountInteraction(CheckboxGroup, {
    props: {
      options: ["a", "b"],
      ariaLabel: "Letters",
      "onUpdate:modelValue": (next: readonly string[]) => {
        selected.value = next;
      },
    },
    slots: { default: renderGroup(["a", "b"]) },
  });
  const api = handle.exposes<CheckboxGroupExpose<string>>();

  assert.equal(api.setSelected("b", true), true);
  assert.equal(api.isSelected("b"), true);
  assert.equal(api.someSelected, true);
  assert.equal(api.setSelected("zzz", true), false, "unknown values are ignored");
  assert.equal(api.setAll(true), true);
  assert.equal(api.allSelected, true);
  assert.equal(api.state, "all");
  assert.deepEqual(selected.value, ["a", "b"]);
  assert.equal(api.reset(), true);
  assert.deepEqual(api.values, []);
  assert.equal(api.state, "none");
  assert.ok(api.root === handle.root());
  handle.unmount();
});

test("items and select-all require a CheckboxGroup provider", () => {
  const warn = console.warn;
  console.warn = () => undefined;
  try {
    assert.throws(
      () => mountInteraction(CheckboxGroupItem, { props: { value: "a" } }),
      /VIZE_UI_CONTEXT_MISSING: CheckboxGroup/,
    );
    assert.throws(() => mountInteraction(CheckboxGroupSelectAll), /VIZE_UI_CONTEXT_MISSING/);
  } finally {
    console.warn = warn;
  }
});
