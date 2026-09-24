import assert from "node:assert/strict";

import { test } from "vite-plus/test";
import { defineComponent, h, nextTick, ref } from "vue";

import { mountInteraction } from "../../../testing/mount.ts";
import type { SelectRootExpose } from "./select.ts";
import SelectContent from "./select-content.vue";
import SelectGroup from "./select-group.vue";
import SelectItem from "./select-item.vue";
import SelectLabel from "./select-label.vue";
import SelectRoot from "./select-root.vue";
import SelectSeparator from "./select-separator.vue";
import SelectTrigger from "./select-trigger.vue";
import SelectValue from "./select-value.vue";
import {
  fruits,
  keydown,
  fruitSelectOptions,
  optionId,
  settle,
  trigger,
} from "./select-test-utils.ts";
import type { Fruit } from "./select-test-utils.ts";

function mountFruitSelect(
  rootProps: Record<string, unknown> = {},
  contentProps: Record<string, unknown> = {},
) {
  return mountInteraction(SelectRoot, fruitSelectOptions(rootProps, contentProps));
}

test("renders select-only combobox semantics with placeholder and closed popup", async () => {
  const handle = mountFruitSelect({ placeholder: "Pick a fruit", required: true });
  await settle();
  const button = trigger(handle);

  assert.equal(handle.root().getAttribute("data-vize-ui"), "select");
  assert.equal(handle.root().getAttribute("data-state"), "closed");
  assert.equal(handle.root().getAttribute("data-empty"), "true");
  assert.equal(button.id, "fruit-trigger");
  assert.equal(button.type, "button");
  assert.equal(button.getAttribute("aria-haspopup"), "listbox");
  assert.equal(button.getAttribute("aria-expanded"), "false");
  assert.equal(button.getAttribute("aria-controls"), null);
  assert.equal(button.getAttribute("aria-required"), "true");
  assert.equal(button.getAttribute("data-placeholder"), "true");
  assert.equal(button.querySelector("[data-vize-ui='select-value']")?.textContent, "Pick a fruit");
  assert.equal(handle.queryByRole("listbox"), null);
  handle.unmount();
});

test("click opens the listbox, highlights the selected option, and click selects", async () => {
  const handle = mountFruitSelect({ defaultValue: fruits[1] });
  const button = trigger(handle);
  await handle.click(button);
  await settle();

  const listbox = handle.getByRole("listbox");
  assert.equal(listbox.id, "fruit-listbox");
  assert.equal(listbox.getAttribute("aria-labelledby"), "fruit-trigger");
  assert.equal(button.getAttribute("aria-expanded"), "true");
  assert.equal(button.getAttribute("aria-controls"), "fruit-listbox");
  assert.equal(button.getAttribute("aria-activedescendant"), optionId(handle, "Banana"));
  const banana = handle.getByRole("option", { name: /^Banana/ });
  assert.equal(banana.getAttribute("aria-selected"), "true");
  assert.equal(banana.getAttribute("data-highlighted"), "true");
  const indicator = banana.querySelector("[data-vize-ui='select-item-indicator']");
  assert.equal(indicator?.hasAttribute("hidden"), false);
  assert.equal(indicator?.textContent, "✓");
  const appleIndicator = handle
    .getByRole("option", { name: "Apple" })
    .querySelector("[data-vize-ui='select-item-indicator']");
  assert.equal(appleIndicator?.hasAttribute("hidden"), true);
  assert.equal(appleIndicator?.textContent, "");

  await handle.click(handle.getByRole("option", { name: "Date" }));
  await settle();
  assert.equal(button.getAttribute("aria-expanded"), "false");
  assert.equal(button.querySelector("[data-vize-ui='select-value']")?.textContent, "Date");
  const changes = handle.recorded().filter((entry) => entry.event === "change");
  assert.deepEqual(
    changes.map((entry) => [entry.payload[0], entry.payload[1]]),
    [[fruits[4], fruits[1]]],
  );
  handle.unmount();
});

test("pointer movement highlights enabled options without selecting", async () => {
  const handle = mountFruitSelect({ defaultOpen: true });
  await settle();
  const button = trigger(handle);
  const date = handle.getByRole("option", { name: "Date" });
  date.dispatchEvent(new PointerEvent("pointermove", { bubbles: true, pointerType: "mouse" }));
  await nextTick();
  assert.equal(button.getAttribute("aria-activedescendant"), date.id);
  assert.equal(date.getAttribute("aria-selected"), "false");

  const cherry = handle.getByRole("option", { name: "Cherry" });
  cherry.dispatchEvent(new PointerEvent("pointermove", { bubbles: true, pointerType: "mouse" }));
  await handle.click(cherry);
  assert.equal(cherry.getAttribute("aria-disabled"), "true");
  assert.equal(button.getAttribute("aria-activedescendant"), date.id);
  assert.equal(handle.recorded().filter((entry) => entry.event === "change").length, 0);

  const pointerdown = new PointerEvent("pointerdown", { bubbles: true, cancelable: true });
  handle.getByRole("listbox").dispatchEvent(pointerdown);
  assert.equal(pointerdown.defaultPrevented, true, "options never steal trigger focus");
  handle.unmount();
});

test("multiple selection toggles values, keeps the popup open, and emits arrays", async () => {
  const handle = mountFruitSelect({ defaultValue: [fruits[0]], multiple: true });
  const button = trigger(handle);
  await handle.click(button);
  await settle();

  const listbox = handle.root().querySelector("[data-vize-ui='select-content']");
  assert.equal(listbox?.getAttribute("role"), "listbox");
  assert.equal(listbox?.getAttribute("aria-multiselectable"), "true");
  await handle.click(handle.getByRole("option", { name: "Banana" }));
  await handle.click(handle.getByRole("option", { name: /^Apple/ }));
  assert.equal(button.getAttribute("aria-expanded"), "true");
  const updates = handle
    .recorded()
    .filter((entry) => entry.event === "update:modelValue")
    .map((entry) => entry.payload[0]);
  assert.deepEqual(updates, [[fruits[0], fruits[1]], [fruits[1]]]);
  assert.equal(Object.isFrozen(updates[0]), true);
  assert.equal(button.querySelector("[data-vize-ui='select-value']")?.textContent, "Banana");
  assert.equal(handle.root().getAttribute("data-selection-mode"), "multiple");
  handle.unmount();
});

test("the attribute form of multiple selects multiple mode", async () => {
  const handle = mountFruitSelect({ multiple: "" });
  await handle.click(trigger(handle));
  await settle();
  await handle.click(handle.getByRole("option", { name: "Date" }));
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[[fruits[4]]]]);
  handle.unmount();
});

test("controlled value and open state wait for the parent", async () => {
  const handle = mountFruitSelect({ modelValue: fruits[0], open: false });
  const button = trigger(handle);
  await handle.click(button);
  assert.deepEqual(handle.wrapper.emitted("update:open"), [[true]]);
  assert.equal(button.getAttribute("aria-expanded"), "false");

  await handle.wrapper.setProps({ open: true });
  await settle();
  await handle.click(handle.getByRole("option", { name: "Date" }));
  assert.deepEqual(handle.wrapper.emitted("update:modelValue"), [[fruits[4]]]);
  assert.equal(button.textContent, "Apple");
  await handle.wrapper.setProps({ modelValue: { id: 5, name: "Date" } });
  assert.equal(button.textContent, "Date");
  assert.equal(
    handle.getByRole("option", { name: /^Date/ }).getAttribute("aria-selected"),
    "true",
    "`by` matches structurally equal values",
  );
  handle.unmount();
});

test("a comparator `by` and object text fallbacks resolve selection and labels", async () => {
  const handle = mountFruitSelect({
    by: (left: Fruit, right: Fruit) => left.name.toLowerCase() === right.name.toLowerCase(),
    defaultValue: { id: 99, name: "banana" },
    itemText: undefined,
  });
  await settle();
  assert.equal(trigger(handle).textContent, "banana");
  await handle.click(trigger(handle));
  await settle();
  assert.equal(
    handle.getByRole("option", { name: /^Banana/ }).getAttribute("aria-selected"),
    "true",
  );
  handle.unmount();
});

test("disabled roots do not open and disabled options are skipped", async () => {
  const handle = mountFruitSelect({ disabled: true });
  const button = trigger(handle);
  assert.equal(button.disabled, true);
  keydown(button, "ArrowDown");
  await settle();
  assert.equal(button.getAttribute("aria-expanded"), "false");
  assert.equal(handle.root().getAttribute("data-disabled"), "true");
  handle.unmount();
});

test("groups wire labels and separators stay out of the accessibility tree", async () => {
  const handle = mountInteraction(SelectRoot, {
    props: { defaultOpen: true, id: "produce" },
    slots: {
      default: () => [
        h(SelectTrigger, { ariaLabel: "Produce" }, () => h(SelectValue)),
        h(SelectContent, { portalDisabled: true }, () => [
          h(SelectGroup, { id: "fruit-group" }, () => [
            h(SelectLabel, null, () => "Fruit"),
            h(SelectItem<string>, { value: "apple" }, () => "Apple"),
          ]),
          h(SelectSeparator),
          h(SelectGroup, { ariaLabel: "Vegetables" }, () => [
            h(SelectItem<string>, { value: "leek" }, () => "Leek"),
          ]),
        ]),
      ],
    },
  });
  await settle();
  const group = handle.getByRole("group", { name: "Fruit" });
  assert.equal(group.getAttribute("aria-labelledby"), "fruit-group-label");
  assert.ok(handle.getByRole("group", { name: "Vegetables" }));
  const separator = handle.root().querySelector("[data-vize-ui='select-separator']");
  assert.equal(separator?.getAttribute("aria-hidden"), "true");
  handle.unmount();
});

test("exposed methods drive the same selection and open state", async () => {
  let exposed: SelectRootExpose<Fruit> | null = null;
  const model = ref<Fruit | null>(null);
  const Probe = defineComponent({
    setup: () => () =>
      h(
        SelectRoot<Fruit>,
        {
          by: "id",
          defaultValue: fruits[2],
          itemText: (fruit: Fruit) => fruit.name,
          modelValue: model.value ?? undefined,
          ref: (value) => {
            exposed = value as SelectRootExpose<Fruit> | null;
          },
        },
        () => [h(SelectTrigger, { ariaLabel: "Fruit" }, () => h(SelectValue))],
      ),
  });
  const handle = mountInteraction(Probe);
  if (exposed === null) assert.fail("SelectRoot must expose state");
  const api: SelectRootExpose<Fruit> = exposed;

  assert.deepEqual(api.selected, [fruits[2]]);
  assert.equal(api.select(fruits[0] as Fruit), true);
  await nextTick();
  assert.deepEqual(api.selectedText, ["Apple"]);
  assert.equal(api.deselect(fruits[0] as Fruit), true);
  await nextTick();
  assert.equal(api.selected.length, 0);
  assert.equal(api.reset(), true);
  await nextTick();
  assert.deepEqual(api.selectedText, ["Blueberry"]);
  assert.equal(api.clear(), true);
  assert.equal(api.setOpen(true), true);
  await nextTick();
  assert.equal(api.open, true);
  api.focus();
  assert.equal(handle.activeElement(), handle.getByRole("combobox", { name: "Fruit" }));
  handle.unmount();
});

test("parts require a Select provider", () => {
  assert.throws(
    () => mountInteraction(SelectItem, { props: { value: "orphan" } }),
    /VIZE_UI_CONTEXT_MISSING/,
  );
});
